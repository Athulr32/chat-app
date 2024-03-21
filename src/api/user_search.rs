use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use jwt::VerifyWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{collections::BTreeMap, sync::Arc};
use surrealdb::Surreal;
use tokio::sync::RwLock;

use hmac::{Hmac, Mac};

use crate::{db::surreal::schema::Users, api::error::CustomError, api::net::HttpResponse, api::types::AppState};


pub fn check_jwt(header: &HeaderMap) -> bool {
    if header.contains_key("AUTHENTICATION") {
        match header["AUTHENTICATION"].to_str() {
            Ok(token) => {
                let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

                let claims: Result<BTreeMap<String, String>, jwt::Error> =
                    token.verify_with_key(&key);
                matches!(claims, Ok(_claim))
            }
            Err(_) => false,
        }
    } else {
        false
    }
}

//Respond with all the users having the name param
#[axum_macros::debug_handler]
pub async fn user_search(
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if check_jwt(&header) {
        let read_app_state = app_state.read().await;
        let get_db_client = read_app_state.get_db_client();
        let get_db_client = get_db_client.read().await;

        let get_users: Result<Vec<Users>, surrealdb::Error> = get_db_client.select("users").await;

        if get_users.is_err() {
            return Err(CustomError::DbError);
        }
        let users = get_users.unwrap();

        return Ok(HttpResponse::json(&users));
    }
    Err(CustomError::WrongDigitalSignature)
}
