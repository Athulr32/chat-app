use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    api::{
        error::CustomError,
        types::{AppState, TypingInfo},
        websocket::convert_to_json,
    },
    types::enums::MessageType,
};

use super::utils::jwt::check_jwt;

//Endpoint to info the receiver that the sender is typing the message
pub async fn typing(
    Path(user_id): Path<String>,
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
        return Err(CustomError::WrongDigitalSignature);
    }

    let (public_key, _name) = jwt_verification.unwrap();

    let app_state = app_state.read().await;
    let state = app_state.clone().get_state();
    let state = state.read().await;

    //Check if receiver is online
    let reciever_socket = state.get(&user_id);

    if let Some(receiver) = reciever_socket {
        //Send typing signal to receiver
        let payload = TypingInfo {
            from: public_key,
            message_type: MessageType::TYPING,
        };
        let _ = receiver.send(convert_to_json(&payload));
    }
    //If not don't do anything

    Ok(())
}
