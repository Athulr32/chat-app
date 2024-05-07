use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{lock::Mutex, SinkExt, StreamExt};
use serde::Serialize;
use serde_json::{json, Value};
use surrealdb::{engine::remote::ws::Client, Surreal};

use std::{
    borrow::BorrowMut,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use hmac::{Hmac, Mac};
use jwt::{Store, VerifyWithKey};
use tokio::sync::{broadcast, RwLock};
use uuid::{uuid, Timestamp, Uuid};

use crate::api::types::{
    AppState, ChatState, ClientAuthWsMessage, ClientPrivateMessage,
    ClientWsMessageInvalidJsonFormat, GetMessage, MessageStatus, RecipientMessage,
    SocketAuthUserMessage,
};
use sha2::Sha256;
use std::collections::BTreeMap;

#[axum_macros::debug_handler]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Response {
    //upgrade the websocket connection
    ws.on_failed_upgrade(|_| {})
        .on_upgrade(move |socket| handle_socket(socket, app_state))
}

async fn handle_socket(socket: WebSocket, state: Arc<RwLock<AppState>>) {
    //Socket instance = TO communicate between users
    //Channel Instance = To communicate between users threads that is spawned for different users

    //Split the socket of the user into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    //Create a channel for communication between users threads
    //it should MPSC because rx will be with this user and tx can be cloned by other users threads to send to rx
    let (tx, mut rx) = broadcast::channel(100);

    //Spwan the sender socket instance into a new thread
    //Also move the rx channel into this
    let receiver_handler = tokio::spawn(async move {
        //Wait for message from the channel
        while let Ok(msg) = rx.recv().await {
            println!("Receieved message to send to client {:?}", msg);
            //If message then send that message to user using the sender socket instance
            let send_to_client = sender.send(Message::Text(msg)).await;

            if send_to_client.is_err() {

                //If sending failed Add the message to database
            }
        }
    });

    //Spawn the Receiver socket instance into a new thread
    //Also move the tx channel into this
    let _sender_handler = tokio::spawn(async move {
        let mut auth = false;
        let mut pk = String::from("");
        let mut name = String::from("");

        //Check for message from the client in receiever socket instance
        while let Some(Ok(socket_message)) = receiver.next().await {
            match socket_message {
                Message::Text(msg) => {
                    let mut app_states = state.write().await;
                    let app_state = app_states.get_state().clone();
                    let db_state = app_states.get_db_client().clone();
                    //Client authentication to Web Socket
                    if !auth {
                        let verify = verify_user_authentication_to_websocket(msg, &db_state).await;
                        if let Err(err) = verify {
                            let payload = convert_to_json(&err);
                            //TODO: Handle Error
                            let _ = tx.send(payload);
                        } else {
                            let (public_key, user_name) = verify.unwrap();
                            pk = public_key.clone();
                            name = user_name;
                            let is_user_already_connected =
                                check_if_user_is_already_connected(&public_key, &app_state).await;

                            if !is_user_already_connected {
                                //Add to State
                                let mut app_state = app_state.write().await;
                                app_state.insert(public_key.clone(), tx.clone());
                                auth = true;
                                let payload = convert_to_json(&ClientAuthWsMessage::new(
                                    "authentication".to_string(),
                                    true,
                                    "Success".to_string(),
                                ));
                                let _ = tx.send(payload);
                            } else {
                                let payload = convert_to_json(&ClientAuthWsMessage::new(
                                    "authentication".to_string(),
                                    false,
                                    "Already Connected Multiple Connection Not Allowed".to_string(),
                                ));
                                let _ = tx.send(payload);
                            }
                        }
                    } else {
                        //If user is authenticated Execute this Block
                        //Client message

                        //Now  there is Message_type field in JSON sent by the client
                        let check_message_type = check_for_proper_message_type(&msg);
                        if let Err(ref err) = check_message_type {
                            let _ = tx.send(err.clone());
                        }

                        let get_message_type = check_message_type.unwrap();

                        match get_message_type.as_str() {
                            "private_message" => {
                                //WANT CLIENT MESSAGE IN THIS FORMAT ONLY FOR PRIVATE MESSAGE
                                let user_message: Result<ClientPrivateMessage, serde_json::Error> =
                                    serde_json::from_str(&msg);

                                if user_message.is_err() {
                                    let payload = convert_to_json(
                                        &ClientWsMessageInvalidJsonFormat::default(),
                                    );
                                    let send_to_client = tx.send(payload);

                                    if let Err(_) = send_to_client {
                                        break;
                                    }

                                    continue;
                                }

                                //Client Message is in Correct Format
                                let client_message = user_message.unwrap();

                                //Recipient public key
                                let rec_pubkey: String = client_message.get_to_public_key();

                                //UID of the message sent by the sender
                                let message_id: String = client_message.get_mesage_id();

                                //Get the Connection HashMap
                                let unlock_state = app_state.read().await;
                                let time = SystemTime::now();
                                let since_the_epoch = time
                                    .duration_since(UNIX_EPOCH)
                                    .expect("Time went backwards");
                                let current_time = since_the_epoch.as_secs() * 1000
                                    + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

                                //Db Client
                                let db_client = db_state.write().await;

                                let message = crate::types::message::Message {
                                    from: pk.clone(),
                                    cipher: client_message.get_cipher(),
                                    message_id: message_id,
                                    to: rec_pubkey.clone(),
                                    time: current_time,
                                    status: crate::db::surreal::schema::UserMessageStatus::Sent,
                                    message_type: String::from("private_message"),
                                    to_name: String::from("Name"), //TODO: TO be Chnaged,
                                    from_name: String::from("Name"), //TODO: TO be Chnaged,
                                    cipher_self: client_message.get_cipher_self(),
                                    info_type: client_message.get_info_type(),
                                };

                                let id = Uuid::new_v4().to_string();
                                let insert_message: Result<
                                    Option<crate::db::surreal::schema::Message>,
                                    surrealdb::Error,
                                > = db_client
                                    .create(("messages", id.clone()))
                                    .content(message)
                                    .await;

                                //Get the socket channel of the recipient  using the public key
                                let transmit_channel_of_recipient = unlock_state.get(&rec_pubkey);

                                if insert_message.is_err() {
                                    //TODO: To be Fixes
                                    continue;
                                }
                                let message_for_receiver = insert_message.unwrap().unwrap();

                                // //Construct message for the recipent and also to add in DB
                                // let message_for_receiver = RecipientMessage::build(
                                //     client_message.get_uid(),
                                //     client_message.message_type.clone(),
                                //     client_message.get_cipher(),
                                //     pk.clone(),
                                //     rec_pubkey.to_string(),
                                //     uid.clone(),
                                //     name.to_string(),
                                //     current_time,
                                // );

                                //If user is online
                                //Send message to user
                                if let Some(tr) = transmit_channel_of_recipient {
                                    let _send_message_to_recipient =
                                        tr.send(convert_to_json(&message_for_receiver));
                                }
                                let message_status = MessageStatus::build(
                                    "status".to_string(),
                                    rec_pubkey,
                                    id,
                                    "sent".to_string(),
                                    "true".to_string(),
                                );
                                let _reply_to_client = tx.send(convert_to_json(&message_status));
                            }

                            "get_message" => {
                                let unlock_db = db_state;
                                let db_client = unlock_db.read().await;
                            }
                            _ => {}
                        }
                    }
                }
                Message::Ping(msg) => {
                    println!("{:?}", msg);
                }
                Message::Pong(msg) => {
                    println!("{:?}", msg);
                }
                Message::Binary(msg) => {
                    println!("{:?}", msg);
                }
                Message::Close(msg) => {
                    println!("{:?}", msg);
                }
            }
        }
        let mut app_states = state.write().await;
        let unlock_state = app_states.get_state();
        let mut unlock_state = unlock_state.write().await;

        unlock_state.remove(&pk[..]);

        receiver_handler.abort();

        println!("Disconnected");
    });
}

pub async fn add_user_to_auth_pool(public_key: &str, state: ChatState) {}

pub async fn check_if_user_is_already_connected(public_key: &str, state: &ChatState) -> bool {
    let app_state = state.read().await;
    let get_connection = app_state.get(public_key);
    get_connection.is_some()
}

pub async fn verify_user_authentication_to_websocket(
    user_messge: String,
    db: &Arc<RwLock<Surreal<Client>>>,
) -> Result<(String, String), ClientAuthWsMessage> {
    //Verify and decode Client JWT TOKEN
    let decode_socket_auth: Result<SocketAuthUserMessage, serde_json::Error> =
        serde_json::from_str(&user_messge);

    if let Ok(auth) = decode_socket_auth {
        let token = auth.get_token();

        let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

        let claims: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);

        if let Ok(claim) = claims {
            //Get User Details From Claims
            //TODO: Some more validation
            let public_key = claim["public_key"].to_string();
            let name = claim["user_name"].to_string();

            Ok((public_key, name))
        } else {
            Err(ClientAuthWsMessage::new(
                "authentication".to_string(),
                false,
                "Invalid JWT".to_string(),
            ))
        }
    } else {
        Err(ClientAuthWsMessage::new(
            String::from("authentication"),
            false,
            String::from("Invalid JSON Format"),
        ))
    }
}

pub fn convert_to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

pub fn is_valid_json(message: &str) -> Result<Value, serde_json::Error> {
    Ok(serde_json::from_str(message)?)
}

pub fn send_error_to_user() {}

pub fn check_for_proper_message_type(message: &str) -> Result<String, String> {
    let user_message = is_valid_json(message);
    //if the message is not in proper json format send error message to client
    if user_message.is_err() {
        let payload = convert_to_json(&ClientWsMessageInvalidJsonFormat::default());

        return Err(payload);
    }

    //WANT message_type field in the JSON if not avaialble send error message
    let client_message: serde_json::Value = user_message.unwrap();

    let message_type = client_message.get("message_type");

    if message_type.is_none() {
        let payload = convert_to_json(&ClientWsMessageInvalidJsonFormat::build());
        return Ok(payload);
    }

    let message_type = message_type.unwrap().as_str();

    if message_type.is_none() {
        let payload = convert_to_json(&ClientWsMessageInvalidJsonFormat::build());

        return Err(payload);
    }

    let message_type = message_type.unwrap().to_string();

    Ok(message_type)
}
