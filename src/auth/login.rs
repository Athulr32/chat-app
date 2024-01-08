// use std::sync::Arc;

// use axum::{extract::State, Json};
// use secp256k1::{Secp256k1, Message, ecdsa::Signature, PublicKey};
// use serde::Serialize;
// use sha2::Sha256;




// use crate::{
//     error::CustomError,
//     types::AppState,
// };


// //JWT
// #[derive(Serialize)]
// pub struct JWT {
//     token: String,
// }

// //User login Details
// #[derive(Serialize, Deserialize)]
// pub struct LoginCredential {
//     signature: Vec<u8>,
//     recid: u8,
//     message: String,
//     pub_key: Vec<u8>,
// }

// impl LoginCredential {
//     //Check Digital Signature
//     fn validate_digital_signature(&self) -> Result<bool, anyhow::Error> {
//         let secp256k1 = Secp256k1::new();

//         let mut hasher = Sha256::new();
//         hasher.update(&self.message);
//         let result = hasher.finalize();

//         let message = Message::from_slice(&result)?;
//         let signature = Signature::from_compact(&self.signature[..])?;
//         let public_key = PublicKey::from_slice(&self.pub_key)?;

//         Ok(secp256k1
//             .verify_ecdsa(&message, &signature, &public_key)
//             .is_ok())
//     }
// }


// pub async fn login(
//     State(app_state): State<Arc<AppState>>,
//     Json(data): Json<LoginCredential>,
// ) -> Result<Json<JWT>, CustomError> {
//     //Check if Digital Signature is Valid
//     let check_ecdsa = data.validate_digital_signature();

//     let db_client = app_state.get_db_client();
// }
