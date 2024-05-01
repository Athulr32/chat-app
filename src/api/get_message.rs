use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    api::{error::CustomError, net::HttpResponse, types::AppState},
    db::surreal::schema::Message,
};

use super::utils::jwt::check_jwt;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    before: Option<String>,
    after: Option<u64>,
    limit: Option<u64>,
}

pub async fn get_message(
    Path(userId): Path<String>,
    query: Query<QueryParams>,
    State(app_state): State<Arc<RwLock<AppState>>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let jwt_verification = check_jwt(&header);

    if jwt_verification.is_err() {
        return Err(CustomError::WrongDigitalSignature);
    }

    let (sender_public_key, _name) = jwt_verification.unwrap();

    let receiver_public_key = userId;

    let app_state = app_state.read().await;
    let db_state = app_state.get_db_client();
    let db_client = db_state.read().await;

    //Get the latest {limit} messages if after and before is empty
    let mut limit: u64 = 50;

    let query_limit = query.limit;

    if let Some(l) = query_limit {
        limit = l
    }

    let mut message_required: Vec<Message> = vec![];

    if let Some(before) = query.before.clone() {
        let messages = db_client
                .query("Select * from messages WHERE (from=$senderpublickey AND to=$receiverpublickey) OR (from=$receiverpublickey AND to=$senderpublickey)  AND id > $id  ORDER BY id ASC LIMIT $limit")
                .bind(("senderpublickey",sender_public_key.clone()))
                .bind(("receiverpublickey",receiver_public_key.clone()))
                .bind(("id",before))
                .bind(("limit",limit))
                .await;

        if messages.is_err() {
            return Err(CustomError::DbError);
        }

        let mut messages = messages.unwrap();
        message_required = messages.take(0).unwrap();
    } else if let Some(before) = query.after.clone() {
        let messages = db_client
                .query("Select * from messages WHERE (from=$senderpublickey AND to=$receiverpublickey) OR (from=$receiverpublickey AND to=$senderpublickey)  AND id < $id  ORDER BY id ASC LIMIT $limit")
                .bind(("senderpublickey",sender_public_key.clone()))
                .bind(("receiverpublickey",receiver_public_key.clone()))
                .bind(("id",before))
                .bind(("limit",limit))
                .await;

        if messages.is_err() {
            return Err(CustomError::DbError);
        }

        let mut messages = messages.unwrap();
        message_required = messages.take(0).unwrap();
    } else {
        //Get Message from Database
        let messages = db_client
            .query("Select * from messages WHERE (from=$senderpublickey AND to=$receiverpublickey) OR (from=$receiverpublickey AND to=$senderpublickey) ORDER BY id DESC LIMIT $limit")
            .bind(("senderpublickey",sender_public_key.clone()))
            .bind(("receiverpublickey",receiver_public_key.clone()))
            .bind(("limit",limit))
            .await;

        if messages.is_err() {
            return Err(CustomError::DbError);
        }

        println!("Sender key {:?}", sender_public_key);
        println!("Receiver key {:?}", receiver_public_key);

        let mut messages = messages.unwrap();
        // println!("MESSAGES {:?}",messages);
        message_required = messages.take(0).unwrap();
    }

    Ok(HttpResponse::json(&message_required))
}
