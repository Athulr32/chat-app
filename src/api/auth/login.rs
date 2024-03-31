use crate::api::error::CustomError;
use crate::api::types::AppState;
use crate::api::utils::jwt::get_token;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use entity::users::Entity as Users;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sea_orm::ColumnTrait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use secp256k1::hashes::{sha256, Hash};
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::RwLock;
use std::collections::BTreeMap;
use std::time::SystemTime;
use std::{str::FromStr, sync::Arc};
use log::Log;



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

        let message = Message::from_hashed_data::<sha256::Hash>(&self.message.as_bytes());

        let signature = Signature::from_compact(&hex::decode(&self.signature).unwrap()).unwrap();
        let public_key = PublicKey::from_slice(&hex::decode(&self.pub_key).unwrap()).unwrap();

        secp256k1
            .verify_ecdsa(&message, &signature, &public_key)
            .is_ok()
    }
}


pub async fn login(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(data): Json<LoginCredential>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    //Check if Digital Signature is Valid
    let check_ecdsa = data.check_digital_signature();

    if check_ecdsa {
        let db_client = app_state.read().await.get_postgres_client();
        let db_client = db_client.read().await;

        //Check if user exist
        let check_public_key_exist = Users::find()
            .filter(entity::users::Column::PublicKey.eq(&data.pub_key))
            .one::<DatabaseConnection>(&db_client)
            .await;

        if let Ok(user) = check_public_key_exist {
            if let Some(user) = user {
                return Ok(get_token(&user.public_key, &user.user_name));
            }

            let error = CustomError::UserNotRegistered {
                message: String::from("User Not registered"),
                status: false,
            };
            return Err(error);
        }

        return Err(CustomError::DbError);
    }
    Err(CustomError::WrongDigitalSignature)
}
