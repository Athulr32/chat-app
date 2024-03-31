use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone, Debug)]
pub enum MessageType {
    PrivateMessage,
    AUTHENTICATION,
    GroupMessage,
    TYPING,
}

impl MessageType {
    pub fn to_string(&self) -> String {
        match self {
            MessageType::AUTHENTICATION => "AUTHENITCATION".to_string(),
            MessageType::GroupMessage => "GROUP_MESSAGE".to_string(),
            MessageType::PrivateMessage => "PRIVATE_MESSAGE".to_string(),
            MessageType::TYPING => "TYPING".to_string(),
        }
    }
}
