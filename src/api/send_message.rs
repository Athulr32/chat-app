use crate::api::utils::get_current_time_in_seconds;
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    api::{error::CustomError, types::AppState},
    db::surreal::schema::Messages,
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

    let (sender_public_key, _name) = jwt_verification.unwrap();

    let states = app_state.read().await;
    let state = states.clone().get_state().clone();
    let db_state = states.clone().get_db_client().clone();
    let state = state.read().await;
    let db_client = db_state.write().await;
    let receiver_public_key = message.get_public_key();

    //Store message in db
    let id = Uuid::new_v4().to_string();
    let ulid = surrealdb::sql::Id::ulid();
    let message = Messages {
        from: sender_public_key,
        cipher: message.get_cipher(),
        message_id: id.clone(),
        to: receiver_public_key.clone(),
        uid: message.get_uid().clone(),
        time: get_current_time_in_seconds(),
        status: crate::db::surreal::schema::UserMessageStatus::Sent,
    };

    let _insert_message: Result<Option<Messages>, surrealdb::Error> = db_client
        .create(("messages", ulid))
        .content(&message)
        .await;

    if let Err(_) = _insert_message {
        return Err(CustomError::DbError);
    }

    //Check if the receiver is online
    let user = state.get_key_value(&receiver_public_key);
    if let Some(user_ws) = user {
        let _ = user_ws.1.send(serde_json::to_string(&message).unwrap());
    }

    Ok(HttpResponse::text(String::from("Done")))
}
