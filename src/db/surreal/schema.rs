use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::api::types::Chain;

#[derive(Serialize, Deserialize,Debug)]
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

#[derive(Serialize, Deserialize,Debug)]
#[serde(rename_all = "camelCase")]
pub struct Messages {
    pub from: String,
    pub to: String,
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
