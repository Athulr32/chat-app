use crate::api::{types::RecipientMessage, utils::get_current_time_in_seconds};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    api::{error::CustomError, types::AppState},
    db::surreal::schema::Message,
};

use super::{net::HttpResponse, types::ClientPrivateMessage, utils::jwt::check_jwt};

//Client can send message using Http protocol
pub async fn send_message(
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
    Json(message): Json<ClientPrivateMessage>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
        return Err(CustomError::WrongDigitalSignature);
    }

    let (sender_public_key, name) = jwt_verification.unwrap();

    let states = app_state.read().await;
    let state = states.clone().get_state().clone();
    let db_state = states.clone().get_db_client().clone();
    let state = state.read().await;
    let db_client = db_state.write().await;
    let receiver_public_key = message.get_to_public_key();
    //Store message in db
    let id = Uuid::new_v4().to_string();
    let ulid = surrealdb::sql::Id::ulid();
    let message = crate::types::message::Message {
        from: sender_public_key.clone(),
        cipher: message.get_cipher(),
        message_id: message.get_mesage_id(),
        to: receiver_public_key.clone(),
        time: get_current_time_in_seconds(),
        status: crate::db::surreal::schema::UserMessageStatus::Sent,
        message_type: String::from("private_message"),
        name: String::from("Athul"), //TODO: TO be Fixed,
    };

    let insert_message: Result<Option<Message>, surrealdb::Error> =
        db_client.create(("messages", ulid)).content(&message).await;

    if let Err(e) = insert_message {
        eprintln!("ERROR:Failed to Insert Message = {:?}", e);
        return Err(CustomError::DbError);
    }

    //Inser to user_chats table
    let _insert_into_user_chats = db_client
        .query("UPDATE chatusers:athul SET chats+=[$receiverpublickey] WHERE COUNT(Select * from chatusers:athul where chats CONTAINS $receiverpublickey)==0")
        .bind(("publickey", sender_public_key.clone()))
        .bind(("receiverpublickey", receiver_public_key.clone()))
        .await;

    if let Err(e) = _insert_into_user_chats {
        eprintln!("ERROR:Failed to Insert into user chats = {:?}", e);
        return Err(CustomError::DbError);
    }

    //Check if the receiver is online
    let user = state.get(&receiver_public_key);
    let payload = insert_message.unwrap().unwrap();
    if let Some(user_ws) = user {
        let _ = user_ws.send(serde_json::to_string(&payload).unwrap());
    }

    Ok(HttpResponse::json(&payload))
}
