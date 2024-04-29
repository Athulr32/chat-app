use std::sync::Arc;

use axum::{
    body::HttpBody,
    routing::{get, post},
    Router,
};
use tokio::sync::RwLock;

use crate::api::types::AppState;

pub mod token;
pub mod transaction;

pub fn router<S, B>(state: Arc<RwLock<AppState>>) -> Router<S, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/send_transaction",
            post(transaction::send_transaction),
        )
        .route(
            "/send_transaction_hash",
            post(transaction::get_transation_hash_from_client),
        )
        .route(
            "/native_token_balance",
            get(token::get_native_token_balance),
        )
        .with_state(state.clone())
}
