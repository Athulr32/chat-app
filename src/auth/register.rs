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

use crate::{db::schema::Users, error::CustomError, types::AppState};
//User Register Details
#[derive(Serialize, Deserialize)]
pub struct RegisterData {
    signature: String,
    message: String,
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

        let message = Message::from_hashed_data::<sha256::Hash>(&self.message.as_bytes());
        let signature = Signature::from_compact(&self.signature.as_bytes()).unwrap();
        let public_key = PublicKey::from_str(&self.pub_key).unwrap();

        secp256k1
            .verify_ecdsa(&message, &signature, &public_key)
            .is_ok()
    }
}

pub async fn get_token(pub_key: &str, name: &str) -> Json<JWT> {
    let system_time = SystemTime::now();
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("public_key", pub_key);
    claims.insert("name", name);
    let token_str = claims.sign_with_key(&key).unwrap();

    Json(JWT { token: token_str })
}

#[axum_macros::debug_handler]
pub async fn register(
    State(client): State<Arc<AppState>>,
    Json(data): Json<RegisterData>,
) -> Result<Json<JWT>, CustomError> {
    let check_ecdsa = data.check_digital_signature();

    if check_ecdsa {
        let unlock_client = client.get_db_client();
        let unlock_client = unlock_client.read().await;

        //Public Key Checks
        let check_public_key_exist: Result<Option<Users>, Error> =
            unlock_client.select(("users", data.pub_key)).await;

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
            .bind(("name", data.name))
            .await;

        if let Ok(mut user) = check_name_exist {
            let get_user: Result<Option<String>, Error> = user.take("name");

            if let Ok(user) = get_user {
                if user.is_none() {
                    //User name is available to register
                } else {
                    //User name already registered
                    return Err(CustomError::UserNameAlreadyExist);
                }
            } else {
                return Err(CustomError::DbError);
            }
        }

        Err(CustomError::SomethingElseWentWrong)
    } else {
        Err(CustomError::WrongDigitalSignature)
    }
}
