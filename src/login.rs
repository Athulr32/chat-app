use crate::error::CustomError;
use crate::types::AppState;
use axum::{extract::State, Json};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::Row;
//JWT
#[derive(Serialize)]
pub struct JWT {
    token: String,
}

//User login Details
#[derive(Serialize, Deserialize)]
pub struct LoginCredential {
    signature: Vec<u8>,
    recid: u8,
    message: String,
    pub_key: Vec<u8>,
}

impl LoginCredential {
    //Check Digital Signature
    fn validate_digital_signature(&self) -> Result<bool, anyhow::Error> {
        let secp256k1 = Secp256k1::new();

        let mut hasher = Sha256::new();
        hasher.update(&self.message);
        let result = hasher.finalize();

        let message = Message::from_slice(&result)?;
        let signature = Signature::from_compact(&self.signature[..])?;
        let public_key = PublicKey::from_slice(&self.pub_key)?;

        Ok(secp256k1
            .verify_ecdsa(&message, &signature, &public_key)
            .is_ok())
    }
}

//To generate JWT TOKEN
pub async fn get_token(pub_key: &str, name: &str) -> Result<Json<JWT>, anyhow::Error> {
    let system_time = SystemTime::now();
    println!("{:?}", pub_key);
    println!("{:?}", system_time);
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd")?;
    let mut claims = BTreeMap::new();
    claims.insert("public_key", pub_key);
    claims.insert("name", name);
    let token_str = claims.sign_with_key(&key)?;

    Ok(Json(JWT { token: token_str }))
}

#[axum_macros::debug_handler]
//User login handler
pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(data): Json<LoginCredential>,
) -> Result<Json<JWT>, CustomError> {
    //Get digital Signature from user and verify
    //If correct issue token containing the public key and time
    print!("{:?}", data.pub_key);

    //Check if time is correct
    let time = SystemTime::now();
    let since_the_epoch = time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let _time_in_milli =
        since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    //Check if Digital Signature is Valid
    let check_ecdsa = data.validate_digital_signature();

    let db_client = app_state.get_db_client();

    let client = db_client.read().await;
    let public_key_in_hex = hex::encode(data.pub_key);

    if let Ok(res) = check_ecdsa {
        if res {
            //Check if user is already in database if not add the user

            let check_user_exist =
                sqlx::query("SELECT name,publicKey from USERS where publicKey=$1")
                    .bind(&public_key_in_hex)
                    .fetch_one(&*client)
                    .await;
            if let Err(err) = check_user_exist {
                match err {
                    sqlx::Error::RowNotFound => {
                        //Add User to Database
                    }
                    _ => return Err(CustomError::DbError),
                }
            } else {
                let user = check_user_exist.unwrap();
                let _user_name: String = user.get(0);
                let _public_key: String = user.get(1);
            }
        } else {
            return Err(CustomError::WrongDigitalSignature);
        }
    } else {
        return Err(CustomError::WrongDigitalSignature);
    }

    let token = get_token(&public_key_in_hex, "Athul").await;

    if let Ok(tok) = token {
        Ok(tok)
    } else {
        Err(CustomError::DbError)
    }


}
