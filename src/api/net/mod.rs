use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;
pub mod generic;
pub enum HttpResponse {
    Json(String),
    Text(String),
}

impl HttpResponse {
    pub fn json<T: Serialize>(value: &T) -> Self {
        Self::Json(serde_json::to_string(value).unwrap())
    }

    pub fn text(value: String) -> Self {
        Self::Text(value)
    }
}

impl IntoResponse for HttpResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            HttpResponse::Json(value) => (StatusCode::ACCEPTED, value).into_response(),
            HttpResponse::Text(value) => (StatusCode::ACCEPTED, value).into_response(),
        }
    }
}

