use std::sync::Arc;

use axum::{
    body::HttpBody,
    extract::State,
    routing::{options, post},
    Router,
};
use tokio::sync::RwLock;

use crate::api::types::AppState;

pub mod register;
pub mod login;
pub fn router<S, B>(state: Arc<RwLock<AppState>>) -> Router<S, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    Router::new()
        .route("/signin", post(register::register))
        .route("/login", post(login::login))
        .with_state(state.clone())
}
