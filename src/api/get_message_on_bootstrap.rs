use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{api::error::CustomError, api::types::AppState};

use super::utils::jwt::check_jwt;

pub async fn get_message_on_boostrap(
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
       return Err(CustomError::WrongDigitalSignature);
    }

    let app_state = app_state.read().await;
    let db_state = app_state.get_db_client();
    let db_client = db_state.read().await;

    Ok(())
}
