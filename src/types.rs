use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::remote::ws::Client};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};

//Client message Model
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ClientPrivateMessage {
    uid: String,              //Id of the message
    pub message_type: String, //Type of the message send by the Client
    cipher: String,           //The encrypted message
    public_key: String,       //Public key of the recipeient
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
    time: String,       // Time at which the client sent the message
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
#[derive(Serialize,Deserialize,Debug)]
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
    state: ChatState, //The state for storing websocket connection
    db_client: Arc<RwLock<Surreal<Client>>>, //Db client state
}

impl AppState {
    pub fn new(
        state: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
        db_client: Arc<RwLock<Surreal<Client>>>,
    ) -> Self {
        AppState { state, db_client }
    }

    pub fn get_state(&mut self) ->  ChatState {
        return self.state.clone();
    }

    pub fn get_db_client(&self) -> Arc<RwLock<Surreal<Client>>> {
        return self.db_client.clone();
    }
}

#[derive(Deserialize, Serialize)]
pub struct GetMessage {
    pub message_type: String,
    pub messages: Vec<RecipientMessage>,
    pub status: bool,
}

impl ClientPrivateMessage {
    pub fn get_public_key(&self) -> String {
        self.public_key.clone()
    }

    pub fn get_uid(&self) -> String {
        self.uid.clone()
    }

    pub fn get_cipher(&self) -> String {
        self.cipher.clone()
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
        time: String,
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
        }
    }

    pub fn get_message_from(&self) -> String {
        return self.from.clone();
    }

    pub fn get_message_to(&self) -> String {
        return self.to.clone();
    }

    pub fn get_message_uid(&self) -> String {
        return self.uid.clone();
    }

    pub fn get_message_type(&self) -> String {
        return self.message_type.clone();
    }

    pub fn get_cipher(&self) -> String {
        return self.cipher.clone();
    }

    pub fn get_message_id(&self) -> String {
        return self.message_id.clone();
    }

    pub fn get_time(&self) -> String {
        return self.time.clone();
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
