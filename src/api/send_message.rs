use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use tokio::sync::RwLock;

use crate::api::{error::CustomError, types::AppState};

use super::types::ClientPrivateMessage;

//Client can send message using Http protocol
pub async fn send_message(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(message):Json<ClientPrivateMessage>
) -> Result<(), impl IntoResponse> {

    


    
    Err(CustomError::DbError)
}
