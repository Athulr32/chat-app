// use axum::{extract::State, http::HeaderMap, Json};
// use jwt::{VerifyWithKey};
// use serde::{Deserialize, Serialize};
// use sha2::Sha256;
// use std::{
//     collections::{BTreeMap},
//     sync::Arc,
// };
// use tokio::sync::{ RwLock};
// use tokio_postgres::Client;

// use hmac::{Hmac, Mac};

// use crate::error::Error;

// #[derive(Serialize, Deserialize)]
// pub struct UserSearchParam {
//     param: String,
//     is_name: bool,
// }

// impl UserSearchParam {
//     pub fn check_jwt(&self, header: &HeaderMap) -> bool {
//         if header.contains_key("AUTHENTICATION") {
//             match header["AUTHENTICATION"].to_str() {
//                 Ok(token) => {
//                     let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

//                     let claims: Result<BTreeMap<String, String>, jwt::Error> =
//                         token.verify_with_key(&key);

//                     // if let Ok(_claim) = claims {
//                     //     true
//                     // } else {
//                     //     false
//                     // }
//                     matches!(claims,Ok(_claim))
//                 }
//                 Err(_) => false,
//             }
//         } else {
//             false
//         }
//     }
// }

// #[axum_macros::debug_handler]
// pub async fn user_search(
//     State(client): State<Arc<RwLock<Client>>>,
//     header: HeaderMap,
//     Json(user): Json<UserSearchParam>,
// ) -> Result<Json<String>, Error> {
//     if user.check_jwt(&header) {
//         let unlock_client = client.read().await;

//         if user.is_name {
//             let check_name_exist = unlock_client
//                 .query(
//                     "SELECT name,publicKey from USERS where name=$1",
//                     &[&user.param],
//                 )
//                 .await;

//             let user = check_name_exist.unwrap();
//             if !user.is_empty() {
//                 let name: &str = user[0].get(1);
//                 println!("{}", name);
//                 Ok(Json(name.to_string()))
//             } else {
//                 Err(Error::UserNotAvailable)
//             }
//         } else {
//             let check_name_exist = unlock_client
//                 .query(
//                     "SELECT name,publicKey from USERS where publicKey=$1",
//                     &[&user.param],
//                 )
//                 .await;

//             let user = check_name_exist.unwrap();
//             if !user.is_empty() {
//                 let name: &str = user[0].get(0);
//                 println!("{}", name);
//                 Ok(Json(name.to_string()))
//             } else {
//                 Err(Error::UserNotAvailable)
//             }
//         }
//     } else {
//         Err(Error::AuthenticationError)
//     }
// }
