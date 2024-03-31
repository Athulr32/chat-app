use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use entity::users::Entity as UsersSample;
use entity::usertokenbalance::Entity as UserTokenBalance;
use entity::usertokens::Entity as UserTokens;
use hex::decode_to_slice;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use log::info;
use sea_orm::{entity::*, query::*, DbErr, TransactionError};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use secp256k1::hashes::{sha256, Hash};
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
//User Register Details
#[derive(Serialize, Deserialize)]
pub struct RegisterData {
    signature: String,
    message: String,
    pub_key: String,
    name: String,
}

use sea_orm::{entity::*, query::*, DbBackend};
use tokio::sync::RwLock;

use crate::api::error::CustomError;
use crate::api::net::HttpResponse;
use crate::api::types::AppState;
use crate::api::utils::jwt::get_token;
//Response to User
#[derive(Serialize, Deserialize)]
pub struct JWT {
    token: String,
}

impl RegisterData {
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

        let postgres_client = client.read().await;
        let postgres_client = postgres_client.get_postgres_client();
        let postgres_client = postgres_client.write().await;

        //Public Key Checks
        let find_public_key_using_postgres = UsersSample::find()
            .filter(entity::users::Column::PublicKey.eq(&data.pub_key))
            .one::<DatabaseConnection>(&postgres_client)
            .await;

        match find_public_key_using_postgres {
            Ok(public_key) => {
                if public_key.is_some() {
                    println!("ERROR: Private Key Already Exist");
                    return Err(CustomError::SomethingElseWentWrong);
                }
                info!("Found Unique Public Key");
            }
            Err(e) => {
                println!("ERROR: Database Error");
                return Err(CustomError::DbError);
            }
        }

        //User Name Checks
        let find_user_using_postgres = UsersSample::find()
            .filter(entity::users::Column::UserName.contains(&data.name))
            .one::<DatabaseConnection>(&postgres_client)
            .await;

        match find_user_using_postgres {
            Ok(user) => {
                if user.is_none() {
                    println!("Unique Registration");
                    let pub_key = data.pub_key.to_string();
                    let name = data.name.to_string();
                    let exec_txn = postgres_client
                        .transaction::<_, (), DbErr>(|db| {
                            Box::pin(async move {
                                let insert_user = UsersSample::insert(entity::users::ActiveModel {
                                    public_key: Set(pub_key),
                                    user_name: Set(name),
                                    ..Default::default()
                                });
                                let user = insert_user.exec(db).await?;

                                let insert_user_tokens =
                                    UserTokens::insert(entity::usertokens::ActiveModel {
                                        user_id: Set(user.last_insert_id),
                                        token_address: Set("Addtssess".to_string()),
                                        token_name: Set("ETssH".to_string()),
                                        ..Default::default()
                                    })
                                    .exec(db)
                                    .await?;

                                let create_token_balance = UserTokenBalance::insert(
                                    entity::usertokenbalance::ActiveModel {
                                        user_tokens_id: Set(insert_user_tokens.last_insert_id),
                                        balance: Set(0),
                                        ..Default::default()
                                    },
                                )
                                .exec(db)
                                .await?;

                                Ok(())
                            })
                        })
                        .await;

                    if let Err(e) = exec_txn {
                        eprintln!("ERROR: {:?}", e);
                        return Err(CustomError::DbError);
                    }
                } else {
                    println!("ERROR : User Name Alredy Exist");
                    return Err(CustomError::UserNameAlreadyExist);
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(CustomError::DbError);
            }
        }
        println!("Successfully Registered New User");
        return Ok(get_token(&data.pub_key, &data.name));
    }
    Err(CustomError::WrongDigitalSignature)
}

#[cfg(test)]
mod tests {
    use hex::decode_to_slice;
    use secp256k1::{ecdsa::Signature, Message, PublicKey};
    use secp256k1::{Keypair, Secp256k1, SecretKey};
    use std::str::FromStr;
    #[test]
    fn check_ds() {
        // let mut secp = Secp256k1::new();
        // let secret_key= SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        // let public_key = secret_key.public_key(&secp);
        // let message = Message::from_digest_slice(&[0xab; 32]).expect("32 bytes");
        // let sig = secp.sign_ecdsa(&message, &secret_key);
        // println!("{:?}",sig);
        let signature = Signature::from_str("05964d101650a11144701047e1dfba172b98910a821ec645f80aa8a6e20dc71b8b654c4cae180e0418eb23a043c47179aa2da1a8c6296945012a2bda7ca726ab").unwrap();
        assert_eq!(1, 2);
    }
}
