use std::{collections::HashMap, sync::Arc};

use futures_util::lock::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
//User message
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ClientMessage {
    uid: String,
    pub message_type: String,
    cipher: String,
    public_key: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MessageStatus {
    message_type: String,
    key: String,
    uid: String,
    status: String,
    message_sent: String,
}

#[derive(Deserialize, Serialize)]
pub struct SocketAuth {
    token: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetMessage {
    pub message_type: String,
    pub messages: Vec<RecieverMessage>,
    pub status: bool,
}

impl ClientMessage {
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

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct RecieverMessage {
    uid: String,
    message_type: String,
    cipher: String,
    public_key: String,
    message_id: String,
    name: String,
    time: String,
}

impl RecieverMessage {
    pub fn build(
        uid: String,
        message_type: String,
        cipher: String,
        public_key: String,
        message_id: String,
        name: String,
        time: String,
    ) -> Self {
        RecieverMessage {
            uid,
            message_type,
            cipher,
            public_key,
            message_id,
            name,
            time,
        }
    }
}

impl MessageStatus {
    pub fn build(
        message_type: String,
        key: String,
        uid: String,
        status: String,
        message_sent: String,
    ) -> Self {
        MessageStatus {
            key,
            message_type,
            uid,
            status,
            message_sent,
        }
    }
}

impl SocketAuth {
    pub fn get_token(&self) -> String {
        self.token.clone()
    }
}

type State = Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>;

#[derive(Clone)]
pub struct AppState {
    state: State,
    db_client: Arc<RwLock<sqlx::SqlitePool>>,
}

impl AppState {
    pub fn new(
        state: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
        db_client: Arc<RwLock<sqlx::SqlitePool>>,
    ) -> Self {
        AppState { state, db_client }
    }

    pub fn get_state(&self) -> State {
        return self.state.clone();
    }

    pub fn get_db_client(&self) -> Arc<RwLock<sqlx::SqlitePool>> {
        return self.db_client.clone();
    }
}
