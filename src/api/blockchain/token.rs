use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::api::{error::CustomError, net::HttpResponse, types::AppState};


#[derive(Serialize,Deserialize)]
pub struct TrackToken{
    token_address:String
}


#[axum_macros::debug_handler]
pub async fn set_token_for_tracking(
    State(client): State<Arc<RwLock<AppState>>>,
    Json(data): Json<TrackToken>
) -> Result<impl IntoResponse, impl IntoResponse> {

    return Ok::<HttpResponse,String>(HttpResponse::text(String::from("Created user")));
}



