use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    api::{error::CustomError, types::AppState},
    db::surreal::schema::Message,
};

use super::{net::HttpResponse, utils::jwt::check_jwt};

pub async fn get_message_on_boostrap(
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
        return Err(CustomError::WrongDigitalSignature);
    }

    let (sender_public_key, _name) = jwt_verification.unwrap();

    let app_state = app_state.read().await;
    let db_state = app_state.get_db_client();
    let db_client = db_state.read().await;

    //Get Message from Database
    let messages = db_client
                .query("Select * from messages WHERE from=$senderpublickey OR  to=$senderpublickey ORDER BY id DESC ")
                .bind(("senderpublickey",sender_public_key.clone()))
                .await;

    if messages.is_err() {
        return Err(CustomError::DbError);
    }

    let mut messages = messages.unwrap();

    let mut message_required: Vec<Message> = messages.take(0).unwrap();

    Ok(HttpResponse::json(
        &message_required
    ))
}
