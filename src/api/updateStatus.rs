// use std::{collections::BTreeMap, sync::Arc};

// use axum::{
//     extract::State,
//     http::HeaderMap,
//     response::{IntoResponse, Response},
//     Json,
// };
// use hmac::{Hmac, Mac};
// use jwt::VerifyWithKey;
// use serde::{Deserialize, Serialize};
// use sha2::Sha256;
// use tokio::sync::RwLock;
// use tokio_postgres::Client;

// use crate::error::Error;

// #[derive(Serialize, Deserialize)]
// pub struct StatusUpdate {
//     //id of the message
//     message_id: String,

//     //Public key of the receiver whose message status should be updated
//     public_key: String,

//     // //What to be updated as status
//     // status:String
// }

// impl StatusUpdate {
//     pub fn check_jwt(&self, header: &HeaderMap) -> bool {
//         if header.contains_key("AUTHENTICATION") {
//             match header["AUTHENTICATION"].to_str() {
//                 Ok(token) => {
//                     let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

//                     let claims: Result<BTreeMap<String, String>, jwt::Error> =
//                         token.verify_with_key(&key);
//                     matches!(claims, Ok(_claim))
//                 }
//                 Err(_) => false,
//             }
//         } else {
//             false
//         }
//     }
// }

// pub async fn update_status_of_message(
//     State(client): State<Arc<RwLock<Client>>>,
//     header: HeaderMap,
//     Json(data): Json<StatusUpdate>,
// ) -> Result<Response, Error> {
//     if header.contains_key("AUTHENTICATION") {
//         match header["AUTHENTICATION"].to_str() {
//             Ok(token) => {
//                 let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

//                 let claims: Result<BTreeMap<String, String>, jwt::Error> =
//                     token.verify_with_key(&key);

//                 if let Ok(claim) = claims {
//                     let client_key = &claim["public_key"];

//                     let db_client = client.read().await;

//                     let update = db_client
//                         .execute(
//                             "UPDATE MESSAGES SET status=$1 WHERE messageFrom=$2 AND messageTo=$3 AND messageid=$4",
//                             &[&"delivered",&data.public_key, &client_key ,&data.message_id],
//                         )
//                         .await;

//                         if update.is_err(){
//                            return Err(Error::DbError)
//                         }

//                         println!("{:?}",update);
//                         Ok("Updated".into_response())
                   
//                 }
//                 else{

//                     Err(Error::AuthenticationError)
//                 }
//             }

//             Err(_) =>  Err(Error::AuthenticationError),
//         }
//     } else {
//         Err(Error::AuthenticationError)
//     }
// }
