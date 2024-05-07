use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tokio::sync::{broadcast, RwLock};

use crate::types::enums::MessageType;

//Data structure to Info the client about typing
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TypingInfo {
    pub message_type: MessageType,
    pub from: String,
}

//Client message Model
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientPrivateMessage {
    message_id: String,       //Id of the message
    cipher: String,           //The encrypted message
    cipher_self:String,
    to: String,               //Public key of the recipient
    pub message_type: String, //Type of the message send by the Client
    pub info_type:String
}

//Recipent Message Model
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct RecipientMessage {
    uid: String,          //UID of the message
    message_type: String, //Type of the message sent
    cipher: String,       //The encrypted form of message
    from: String,         //Public key of the Sender
    to: String,
    message_id: String, //Message id
    name: String,       //Name of the sender (From blockchain naming)
    time: u64,          // Time at which the client sent the message
    info_type:String
}

//Status of each Message sent by the client
#[derive(Deserialize, Debug, Serialize)]
pub struct MessageStatus {
    message_type: String,  //Type of the message (It will be status )
    recipient_key: String, //Inform the status of which chat using the recipient public key
    uid: String,           //UID of the message for which this status is for
    status: String,        //Status of the message Sent,Delivered,Seen (This will be an ENUM)
    message_sent: String, //States whether the message sent by the user is atleast stored in the database or the recipient got it
}

//User Auth Types websocket message
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientAuthWsMessage {
    message_type: String, //Type of the Socket message
    status: bool,         //Whether authenticated or not
    message: String,      //Some messages

}

//WebSocket Authentication Type
#[derive(Deserialize, Serialize)]
pub struct SocketAuthUserMessage {
    token: String, //The jwt token sent by the client to authenticate to the websocket
}

pub type ChatState = Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>;
//State of the App
#[derive(Clone)]
pub struct AppState {
    state: ChatState,                        //The state for storing websocket connection
    db_client: Arc<RwLock<Surreal<Client>>>, //Db client state
    postgres_client: Arc<RwLock<DatabaseConnection>>, // Postgres State
    redis_client: Arc<RwLock<redis::aio::MultiplexedConnection>>,
}

impl AppState {
    pub fn new(
        state: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
        db_client: Arc<RwLock<Surreal<Client>>>,
        postgres_client: Arc<RwLock<DatabaseConnection>>,
        redis_client: Arc<RwLock<redis::aio::MultiplexedConnection>>,
    ) -> Self {
        AppState {
            state,
            db_client,
            postgres_client,
            redis_client,
        }
    }

    pub fn get_state(&mut self) -> ChatState {
        self.state.clone()
    }

    pub fn get_db_client(&self) -> Arc<RwLock<Surreal<Client>>> {
        self.db_client.clone()
    }

    pub fn get_postgres_client(&self) -> Arc<RwLock<DatabaseConnection>> {
        self.postgres_client.clone()
    }

    pub fn get_redis_client(&self) -> Arc<RwLock<redis::aio::MultiplexedConnection>> {
        self.redis_client.clone()
    }
}

#[derive(Deserialize, Serialize)]
pub struct GetMessage {
    pub message_type: String,
    pub messages: Vec<RecipientMessage>,
    pub status: bool,
}

impl ClientPrivateMessage {
    pub fn get_to_public_key(&self) -> String {
        self.to.clone()
    }

    pub fn get_mesage_id(&self) -> String {
        self.message_id.clone()
    }

    pub fn get_cipher(&self) -> String {
        self.cipher.clone()
    }
    pub fn get_cipher_self(&self) -> String {
        self.cipher_self.clone()
    }
    pub fn get_info_type(&self) -> String {
        self.info_type.clone()
    }
}

impl RecipientMessage {
    pub fn build(
        uid: String,
        message_type: String,
        cipher: String,
        from: String,
        to: String,
        message_id: String,
        name: String,
        time: u64,
        info_type:String
    ) -> Self {
        RecipientMessage {
            uid,
            message_type,
            cipher,
            from,
            to,
            message_id,
            name,
            time,
            info_type
        }
    }

    pub fn get_message_from(&self) -> String {
        self.from.clone()
    }

    pub fn get_message_to(&self) -> String {
        self.to.clone()
    }

    pub fn get_message_uid(&self) -> String {
        self.uid.clone()
    }

    pub fn get_message_type(&self) -> String {
        self.message_type.clone()
    }

    pub fn get_cipher(&self) -> String {
        self.cipher.clone()
    }

    pub fn get_message_id(&self) -> String {
        self.message_id.clone()
    }

    pub fn get_time(&self) -> u64 {
        self.time
    }
}

impl MessageStatus {
    pub fn build(
        message_type: String,
        recipient_key: String,
        uid: String,
        status: String,
        message_sent: String,
    ) -> Self {
        MessageStatus {
            recipient_key,
            message_type,
            uid,
            status,
            message_sent,
        }
    }
}

impl SocketAuthUserMessage {
    pub fn get_token(&self) -> String {
        self.token.clone()
    }
}

impl ClientAuthWsMessage {
    pub fn new(message_type: String, status: bool, message: String) -> Self {
        Self {
            message_type,
            status,
            message,
        }
    }
}

#[derive(Serialize)]
pub struct ClientWsMessageInvalidJsonFormat {
    message_type: String,
    status: bool,
    message: String,
}

impl Default for ClientWsMessageInvalidJsonFormat {
    fn default() -> Self {
        ClientWsMessageInvalidJsonFormat {
            message_type: "message_format".to_string(),
            status: false,
            message: "Invalid JSON format".to_string(),
        }
    }
}

impl ClientWsMessageInvalidJsonFormat {
    pub fn build() -> Self {
        Self {
            message: "include message_type in the json".to_string(),
            ..Default::default()
        }
    }
}

enum Status {
    Sent,
    Delivered,
    Seen,
}

#[derive(Serialize, Deserialize)]
pub enum Chain {
    Ethereum,
}
