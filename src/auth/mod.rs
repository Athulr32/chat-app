use std::sync::Arc;

use axum::{
    body::HttpBody,
    extract::State,
    routing::{options, post},
    Router,
};

use crate::types::AppState;

pub mod register;

pub fn router<S, B>(state: Arc<AppState>) -> Router<S, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    Router::new()
        .route("/signin", post(register::register))
        .with_state(state)
}
