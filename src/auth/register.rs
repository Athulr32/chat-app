use axum::response::IntoResponse;
use axum::{extract::State, Json};
use jwt::SignWithKey;
use secp256k1::hashes::{sha256, Hash};
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{collections::BTreeMap, str::FromStr, sync::Arc, time::SystemTime};
use surrealdb::{engine::remote::ws::Client, Error, Surreal};
use tokio::sync::RwLock;

use hmac::{Hmac, Mac};

use crate::net::HttpResponse;
use crate::{db::schema::Users, error::CustomError, types::AppState};
//User Register Details
#[derive(Serialize, Deserialize)]
pub struct RegisterData {
    signature: String,
    message: u64,
    pub_key: String,
    name: String,
}

//Response to User
#[derive(Serialize, Deserialize)]
pub struct JWT {
    token: String,
}

impl RegisterData {
    fn check_digital_signature(&self) -> bool {
        let secp256k1 = Secp256k1::new();

        let message =
            Message::from_hashed_data::<sha256::Hash>(&self.message.to_string().as_bytes());

        let signature = Signature::from_str(&self.signature).unwrap();
        println!("{:?}", signature);
        let public_key = PublicKey::from_str(&self.pub_key).unwrap();

        secp256k1
            .verify_ecdsa(&message, &signature, &public_key)
            .is_ok()
    }
}

#[axum_macros::debug_handler]
pub async fn register(
    State(client): State<Arc<RwLock<AppState>>>,
    Json(data): Json<RegisterData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let check_ecdsa = data.check_digital_signature();

    if check_ecdsa {
        let unlock_client = client.read().await;
        let unlock_client = unlock_client.get_db_client();
        let unlock_client = unlock_client.read().await;

        //Public Key Checks
        let check_public_key_exist: Result<Option<Users>, Error> =
            unlock_client.select(("users", &data.pub_key)).await;

        if let Ok(user) = check_public_key_exist {
            if user.is_some() {
                return Err(CustomError::UserAlreadyExist);
            }
        } else {
            return Err(CustomError::DbError);
        }

        //User Name Checks
        let check_name_exist = unlock_client
            .query("Select name from users where name=$name")
            .bind(("name", &data.name))
            .await;

        if let Ok(mut user) = check_name_exist {
            let get_user: Result<Option<String>, Error> = user.take("name");

            if let Ok(user) = get_user {
                if user.is_none() {
                    //User name is available to register
                    let register_user: Result<Option<Users>, surrealdb::Error> = unlock_client
                        .create(("users", &data.pub_key))
                        .content(Users {
                            name: data.name.clone(),
                            public_key: data.pub_key.clone(),
                        })
                        .await;

                    if register_user.is_err() {
                        return Err(CustomError::SomethingElseWentWrong);
                    } else {
                        return Ok(HttpResponse::text(String::from("Created user")));
                    }
                } else {
                    //User name already registered
                    return Err(CustomError::UserNameAlreadyExist);
                }
            } else {
                return Err(CustomError::DbError);
            }
        }

        return Err(CustomError::SomethingElseWentWrong);
    } else {
        return Err(CustomError::WrongDigitalSignature);
    }
}
