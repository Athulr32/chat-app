use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    api::{
        error::CustomError,
        types::{AppState, TypingInfo},
        websocket::convert_to_json,
    },
    types::{enums::MessageType, message::SocialMediaMessage},
};

use super::utils::{get_current_time_in_seconds, jwt::check_jwt};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialMessage {
    message: String,
}


pub async fn send_message_public(
    State(state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
    Json(message): Json<SocialMessage>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
        return Err(CustomError::WrongDigitalSignature);
    }

    let (public_key, _name) = jwt_verification.unwrap();

    let state_lock = state.read().await;
    let surreal_client = state_lock.get_db_client();
    let surreal_client = surreal_client.read().await;

    let id = Uuid::new_v4().to_string();
    let ulid = surrealdb::sql::Id::ulid();

    let payload = SocialMediaMessage {
        cipher: message.message,
        from: public_key,
        message_id: ulid.to_string(),
        uid: id,
        time: get_current_time_in_seconds(),
    };

    let insert_message: Result<
        Option<crate::db::surreal::schema::SocialMediaMessage>,
        surrealdb::Error,
    > = surreal_client
        .create(("public_message", ulid))
        .content(&payload)
        .await;

    if insert_message.is_err() {
        return Err(CustomError::DbError);
    }

    Ok(())
}
