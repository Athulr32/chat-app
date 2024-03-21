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
    #[error("User Not Registered")]
    UserNotRegistered { message: String, status: bool },
}

//Impl IntoResponse for the Error
impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        match self {
            CustomError::WrongDigitalSignature => {
                (StatusCode::BAD_REQUEST, "Wrong Digital Signature").into_response()
            }

            CustomError::DbError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error").into_response()
            }
            CustomError::UserAlreadyExist => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error").into_response()
            }
            CustomError::SomethingElseWentWrong => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error").into_response()
            }
            CustomError::UserNameAlreadyExist => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error").into_response()
            }
            CustomError::UserNotRegistered { message, status } => {
                let payload = json!({
                        "message":message,
                        "status":status
                });
                (StatusCode::BAD_REQUEST, Json(payload)).into_response()
            }
        }
    }
}
