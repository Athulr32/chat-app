use crate::api::utils::jwt::check_jwt;
use crate::api::{error::CustomError, net::HttpResponse, types::AppState};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use entity::nativetokenbalance::Entity as UserNativeTokenBalance;
use sea_orm::{entity::*, query::*};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceQuery {
    token_name: String,
    per_page: usize,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Balance {
    token_name: String,
    balance: i64,
    public_key: String,
}

#[axum_macros::debug_handler]
pub async fn get_native_token_balance(
    State(state): State<Arc<RwLock<AppState>>>,
    query: Query<TokenBalanceQuery>,
    header: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
        return Err(CustomError::WrongDigitalSignature);
    }

    let (public_key, _name) = jwt_verification.unwrap();
    let token_to_search = query.token_name.to_string();

    let state_lock = state.read().await;
    let postgres_client = state_lock.get_postgres_client();
    let postgres_client = postgres_client.read().await;

    let balance = UserNativeTokenBalance::find()
        .filter(entity::nativetokenbalance::Column::PublicKey.eq(public_key))
        .filter(entity::nativetokenbalance::Column::PublicKey.eq(token_to_search))
        .one::<DatabaseConnection>(&postgres_client)
        .await;

    if let Ok(balance) = balance {
        match balance {
            Some(bal) => {
                let payload = Balance {
                    balance: bal.token_balance,
                    public_key: bal.public_key,
                    token_name: bal.token_name,
                };
                return Ok(HttpResponse::json(&payload));
            }
            None => return Err(CustomError::UserNameAlreadyExist),
        }
    } else {
        return Err(CustomError::DbError);
    }
}
