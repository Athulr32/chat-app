use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::api::types::Chain;

#[derive(Serialize, Deserialize)]
pub enum UserMessageStatus{
    Sent,
    Received,
    Seen
}
#[derive(Serialize, Deserialize)]
pub struct Users {
    pub public_key: String,
    pub name: String,

}

#[derive(Serialize, Deserialize)]
pub struct Messages {
    pub from_public_key: String,
    pub to_public_key: String,
    pub cipher: String,
    pub message_id: String,
    pub uid: String,
    pub time: u64,
    pub status:UserMessageStatus
}

#[derive(Serialize, Deserialize)]
pub struct UserTokenBalance{
    pub address:String,
    pub token_name:String,
    pub blockchain:Chain,
    pub balance:usize
}
