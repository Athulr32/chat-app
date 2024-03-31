use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;


use crate::{
    api::error::CustomError, api::types::AppState,
};

use super::utils::jwt::check_jwt;

pub fn update_message_status(
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
       return Err(CustomError::WrongDigitalSignature);
    }

    let (public_key,name) = jwt_verification.unwrap();

    Ok(())
}
