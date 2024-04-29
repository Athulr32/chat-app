use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Bytes, TransactionRequest},
    utils::rlp::Rlp,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::api::{error::CustomError, net::HttpResponse, types::AppState};

#[derive(Serialize, Deserialize)]
pub struct TxData {
    data: Vec<u8>,
}

#[axum_macros::debug_handler]
pub async fn send_transaction(
    State(client): State<Arc<RwLock<AppState>>>,
    Json(data): Json<TxData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let rlp_encoded = Rlp::new(&data.data);
    let decode_rlp = TransactionRequest::decode_signed_rlp(&rlp_encoded);

    match decode_rlp {
        Ok(rlp) => {
            //Send the transaction to Blockchain
            let client = Provider::<Http>::try_from("https://eth.drpc.org").unwrap();
            let bytes = data.data;
            let tx_hash = client.send_raw_transaction(Bytes::from(bytes)).await;

            if let Ok(hash) = tx_hash {
                //Save the Transaction Details to SurrealDb Messages
                return Ok(());
            } else {
                return Err(CustomError::SomethingElseWentWrong);
            }
        }
        Err(_) => return Err(CustomError::SomethingElseWentWrong),
    }
}

#[axum_macros::debug_handler]
pub async fn get_transation_hash_from_client(
    State(client): State<Arc<RwLock<AppState>>>,
    Json(data): Json<TxData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    return Ok::<HttpResponse, String>(HttpResponse::text(String::from("Created user")));
}
