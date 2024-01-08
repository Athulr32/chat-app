use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Wrong Digital Signature")]
    WrongDigitalSignature,

    #[error("Server Error")]
    DbError,
    #[error("User already Exist")]
    UserAlreadyExist,
    #[error("User name already Exist")]
    UserNameAlreadyExist,
    #[error("User already Exist")]
    SomethingElseWentWrong,
}

//Impl IntoResponse for the Error
impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            CustomError::WrongDigitalSignature => {
                (StatusCode::BAD_REQUEST, "Wrong Digital Signature")
            }

            CustomError::DbError => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),
            CustomError::UserAlreadyExist => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),
            CustomError::SomethingElseWentWrong => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),
            CustomError::UserNameAlreadyExist => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),
        };

        let payload = json!({
            "error": message,
        });

        (status, Json(payload)).into_response()
    }
}
