use serde::{Deserialize, Serialize};

use crate::db::surreal::schema::UserMessageStatus;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub message_id: String,        //Message Id
    pub from: String,              // Public Key/Address of the Sender
    pub to: String,                // Public Key/Address of the Receiver
    pub cipher: String,            //Encrypted Message
    pub cipher_self: String,            //Encrypted Message
    pub message_type: String,      // Type of the Messgae
    pub time: u64,                 //Time at which the Message has been sent
    pub status: UserMessageStatus, // What's the Status of the Message (Seem,Delivered,Sent)
    pub from_name:String,
    pub to_name:String,
    pub info_type:String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SocialMediaMessage {
    pub from: String,
    pub cipher: String,
    pub message_id: String,
    pub uid: String,
    pub time: u64,
}
