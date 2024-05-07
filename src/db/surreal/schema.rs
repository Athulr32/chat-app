use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::api::types::Chain;

#[derive(Serialize, Deserialize, Debug)]
pub enum UserMessageStatus {
    Sent,
    Received,
    Seen,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Thing,                 // SurrealDb Unique Id
    pub message_id: String,        //Message Id Given by the client TODO: TO be Fixed
    pub from: String,              // Public Key/Address of the Sender
    pub to: String,                // Public Key/Address of the Receiver
    pub from_name:String,
    pub to_name:String,
    pub cipher: String,            //Encrypted Message
    pub cipher_self:String,
    pub message_type: String,      // Type of the Messgae
    pub time: u64,                 //Time at which the Message has been sent
    pub status: UserMessageStatus, // What's the Status of the Message (Seem,Delivered,Sent),
    pub info_type:String
}

#[derive(Serialize, Deserialize)]
pub struct UserChats {
    pub chats: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SocialMediaMessage {
    pub id: Thing,
    pub from: String,
    pub cipher: String,
    pub message_id: String,
    pub uid: String,
    pub time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserAffinity{

}