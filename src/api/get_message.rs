// use std::{collections::BTreeMap, sync::Arc};

// use crate::{
//     error::Error,
//     types::{ClientMessage, RecieverMessage},
// };
// use axum::{
//     extract::State,
//     http::HeaderMap,
//     response::{IntoResponse, Response},
//     Json,
// };
// use futures_util::TryStreamExt;
// use serde::{Deserialize, Serialize};
// use serde_json::json;
// use sha2::Sha256;
// use tokio::sync::RwLock;
// use tokio_postgres::Client;

// use hmac::{Hmac, Mac};
// use jwt::VerifyWithKey;

// #[derive(Serialize, Deserialize)]
// struct NoMessage {
//     status:bool
// }

// #[axum_macros::debug_handler]
// pub async fn get_message(
//     State(client): State<Arc<RwLock<Client>>>,
//     header: HeaderMap,
// ) -> Result<Response, Error> {
//     if header.contains_key("AUTHENTICATION") {
//         match header["AUTHENTICATION"].to_str() {
//             Ok(token) => {
//                 let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

//                 let claims: Result<BTreeMap<String, String>, jwt::Error> =
//                     token.verify_with_key(&key);

//                 if let Ok(claim) = claims {
//                     let client_key = &claim["public_key"];

//                     let unlock_client = client.read().await;

//                     let get_all_client_messages: Result<Vec<tokio_postgres::Row>, tokio_postgres::Error> = unlock_client
//                         .query("SELECT * from messages where messageTo=$1 AND status=$2 ", &[&client_key,&"sent"])
//                         .await;
//                     println!("{:?}",get_all_client_messages);
//                     if get_all_client_messages.is_err() {
//                         return Err(Error::DbError);
//                     } else {
//                         let messages = get_all_client_messages.unwrap();

//                         if messages.is_empty() {
//                             return Ok(Json(NoMessage{status:false}).into_response());
//                         }

//                         let mut foo =Vec::new();
//                         for message in messages {
//                             let message_from: &str = message.get(0);
//                             let message_to: &str = message.get(1);
//                             let message_cipher: &str = message.get(2);
//                             let status: &str = message.get(3);
//                             let message_id: &str = message.get(4);
//                             let time: &str = message.get(5);

//                             let build_message = RecieverMessage::build(
//                                 "f0".to_string(),
//                                 "status".to_string(),
//                                 message_cipher.to_string(),
//                                 message_from.to_string(),
//                                 message_id.to_string(),
//                                 "wfwds".to_string(),
//                                 time.to_string(),
//                             );

//                                 foo.push(build_message);
//                         }

//                         return Ok(Json(foo).into_response());
//                     }
//                 } else {
//                     return Err(Error::AuthenticationError);
//                 }
//             }
//             Err(_) => {
//                 return Err(Error::AuthenticationError);
//             }
//         }
//     }

//     Err(Error::AuthenticationError)
// }
