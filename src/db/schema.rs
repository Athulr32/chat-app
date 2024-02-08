use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Users {
    pub public_key: String,
    pub name: String,
}
