use std::{collections::BTreeMap, time::SystemTime};

use axum::{http::HeaderMap, Json};
use hmac::{Hmac, Mac};
use jwt::{claims, SignWithKey, Store, VerifyWithKey};
use sha2::Sha256;

pub fn check_jwt(header: &HeaderMap) -> Result<(String, String), ()> {
    if header.contains_key("AUTHENTICATION") {
        match header["AUTHENTICATION"].to_str() {
            Ok(token) => {
                let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

                let claims: Result<BTreeMap<String, String>, jwt::Error> =
                    token.verify_with_key(&key);

                if let Ok(claim) = claims {
                    println!("{:?}",claim);
                    let public_key = claim.get("public_key");
                    let name = claim.get("user_name");

                    if public_key.is_none() || name.is_none() {
                        return Err(());
                    }

                    return Ok((public_key.unwrap().to_string(), name.unwrap().to_string()));
                } else {
                    return Err(());
                }
            }
            Err(_) => Err(()),
        }
    } else {
        Err(())
    }
}


#[derive(serde::Serialize)]
pub struct JWT {
    pub token: String,
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
