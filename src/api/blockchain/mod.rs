use std::sync::Arc;

use axum::{body::HttpBody, routing::post, Router};
use tokio::sync::RwLock;

use crate::api::types::AppState;



pub mod transaction;
pub mod token;
pub fn router<S, B>(state: Arc<RwLock<AppState>>) -> Router<S, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    Router::new()
        .route("/send_transaction", post(transaction::send_transaction))
        .route("/send_transaction_hash", post(transaction::get_transation_hash_from_client))
        .route("/set_token_for_tracking", post(token::set_token_for_tracking))
        .with_state(state.clone())
}
