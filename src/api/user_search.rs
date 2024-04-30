use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use entity::users::Entity as Users;
use sea_orm::{EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::utils::jwt::check_jwt;
use crate::{
    api::{error::CustomError, net::HttpResponse, types::AppState},
    types::user::User,
};
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;

#[derive(Serialize, Deserialize)]
pub struct UserSearch {
    name: String,
}

//Respond with all the users having the name param
#[axum_macros::debug_handler]
pub async fn user_search(
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
    Json(search): Json<UserSearch>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);
    let name_to_search = search.name;
    if jwt_verification.is_err() {
        return Err(CustomError::WrongDigitalSignature);
    }

    let read_app_state = app_state.read().await;
    let get_db_client = read_app_state.get_postgres_client();
    let db = get_db_client.read().await;
    let filteredUsers = Users::find()
        .filter(entity::users::Column::UserName.contains(name_to_search))
        .all::<DatabaseConnection>(&db)
        .await;

    if filteredUsers.is_err() {
        return Err(CustomError::DbError);
    }
    let mut temp_user: Vec<User> = Vec::new();
    let users = filteredUsers.unwrap().into_iter().for_each(|user| {
        temp_user.push(User {
            name: user.user_name,
            public_key: user.public_key,
        })
    });

    Ok(HttpResponse::json(&temp_user))
}
