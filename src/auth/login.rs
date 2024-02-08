use std::collections::BTreeMap;
use std::time::SystemTime;
use std::{str::FromStr, sync::Arc};

use crate::db::schema::Users;
use crate::error::CustomError;
use crate::net::HttpResponse;
use crate::types::AppState;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use secp256k1::hashes::{sha256, Hash};
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

//JWT
#[derive(Serialize)]
pub struct JWT {
    pub token: String,
}

//User login Details
#[derive(Serialize, Deserialize)]
pub struct LoginCredential {
    signature: String,
    message: String,
    pub_key: String,
}

impl LoginCredential {
    //Check Digital Signature
    fn check_digital_signature(&self) -> bool {
        let secp256k1 = Secp256k1::new();

        let message =
            Message::from_hashed_data::<sha256::Hash>(&self.message.to_string().as_bytes());

        let signature = Signature::from_str(&self.signature).unwrap();

        let public_key = PublicKey::from_str(&self.pub_key).unwrap();

        secp256k1
            .verify_ecdsa(&message, &signature, &public_key)
            .is_ok()
    }
}

pub fn get_token(pub_key: &str, name: &str) -> Json<JWT> {
    let system_time = SystemTime::now();
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("public_key", pub_key);
    claims.insert("user_name", name);
    let token_str = claims.sign_with_key(&key).unwrap();

    Json(JWT { token: token_str })
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(data): Json<LoginCredential>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    //Check if Digital Signature is Valid
    let check_ecdsa = data.check_digital_signature();

    if check_ecdsa {
        let db_client = app_state.get_db_client();
        let db_client = db_client.read().await;

        //Check if user exist
        let check_public_key_exist: Result<Option<Users>, surrealdb::Error> =
            db_client.select(("users", &data.pub_key)).await;

        if let Ok(user) = check_public_key_exist {
            if let Some(user_details) = user {
                let user_pub_key = user_details.public_key;
                let user_name = user_details.name;

                return Ok(get_token(&user_pub_key, &user_name));
            } else {
                let error = CustomError::UserNotRegistered {
                    message: String::from("User Not registered"),
                    status: false,
                };
                //User Not Registered
                return Err(error);
            }
        } else {
            return Err(CustomError::DbError);
        }
    } else {
        return Err(CustomError::WrongDigitalSignature);
    }
}
