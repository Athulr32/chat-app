use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    api::{error::CustomError, net::HttpResponse, types::AppState}, types::user::User,

};

use super::utils::jwt::check_jwt;

//Respond with all the users having the name param
#[axum_macros::debug_handler]
pub async fn user_search(
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
        return Err(CustomError::WrongDigitalSignature);
    }

    let read_app_state = app_state.read().await;
    let get_db_client = read_app_state.get_db_client();
    let get_db_client = get_db_client.read().await;

    let get_users: Result<Vec<User>, surrealdb::Error> = get_db_client.select("users").await;

    if get_users.is_err() {
        return Err(CustomError::DbError);
    }
    let users = get_users.unwrap();

     Ok(HttpResponse::json(&users))
}
