use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::api::{error::CustomError, net::HttpResponse, types::AppState};


#[derive(Serialize,Deserialize)]
pub struct TxData{
    data:String
}

#[axum_macros::debug_handler]
pub async fn send_transaction(
    State(client): State<Arc<RwLock<AppState>>>,
    Json(data): Json<TxData>
) -> Result<impl IntoResponse, impl IntoResponse> {

    return Ok::<HttpResponse,String>(HttpResponse::text(String::from("Created user")));
}



#[axum_macros::debug_handler]
pub async fn get_transation_hash_from_client(
    State(client): State<Arc<RwLock<AppState>>>,
    Json(data): Json<TxData>
) -> Result<impl IntoResponse, impl IntoResponse> {

    return Ok::<HttpResponse,String>(HttpResponse::text(String::from("Created user")));
}
