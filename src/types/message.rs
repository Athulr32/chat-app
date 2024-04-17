use serde::{Serialize,Deserialize};

use crate::db::surreal::schema::UserMessageStatus;

#[derive(Serialize, Deserialize,Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub from: String,
    pub to: String,
    pub cipher: String,
    pub message_id: String,
    pub uid: String,
    pub time: u64,
    pub status:UserMessageStatus
}