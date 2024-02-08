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

use tokio::sync::{broadcast, RwLock};

use hmac::{Hmac, Mac};
use jwt::{Store, VerifyWithKey};

use crate::types::{
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
            let mut app_states = state.write().await;
            let app_state = app_states.get_state().clone();
            let db_state = app_states.get_db_client().clone();
            match socket_message {
                Message::Text(msg) => {
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

                                //Client Message in Correct Format
                                let client_message = user_message.unwrap();

                                //Recipient public key
                                let rec_pubkey: String = client_message.get_public_key();

                                //UID of the message sent by the sender
                                let uid: String = client_message.get_uid();

                                //Get the Connection HashMap

                                let unlock_state = app_state.read().await;

                                let time = SystemTime::now();
                                let since_the_epoch = time
                                    .duration_since(UNIX_EPOCH)
                                    .expect("Time went backwards");
                                let current_time = since_the_epoch.as_secs() * 1000
                                    + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

                                //Construct message for the recipent and also to add in DB
                                let message_for_receiver = RecipientMessage::build(
                                    client_message.get_uid(),
                                    client_message.message_type.clone(),
                                    client_message.get_cipher(),
                                    pk.clone(),
                                    rec_pubkey.to_string(),
                                    "fd".to_string(),
                                    name.to_string(),
                                    current_time.to_string(),
                                );

                                //Get the socket channel of the recipient  using the public key
                                let transmit_channel_of_recipient = unlock_state.get(&rec_pubkey);

                                //Db Client
                                let db_client = db_state;

                                //If user is online
                                //Send message to user
                                if let Some(tr) = transmit_channel_of_recipient {
                                    let send_message_to_recipient = tr.send(
                                        serde_json::to_string(&message_for_receiver).unwrap(),
                                    );

                                    //Possible chance of user disconnecting at the time of sending the message
                                    //if Error store the message in DB
                                    if send_message_to_recipient.is_err() {
                                        //Add to DB
                                        // let add_to_db = add_message_to_database(
                                        //     &db_client,
                                        //     &message_for_receiver,
                                        //     "Sent",
                                        // )
                                        // .await;

                                        // if add_to_db.is_err() {

                                        //     //Disconnect Websocket and send server error response to client
                                        // }

                                        //Reply to the Client that user is offline and message status is sent
                                        let reply_to_client = tx.send(
                                            serde_json::to_string(&MessageStatus::build(
                                                "status".to_string(),
                                                rec_pubkey,
                                                uid,
                                                "sent".to_string(),
                                                "true".to_string(),
                                            ))
                                            .unwrap(),
                                        );

                                        if let Err(_) = reply_to_client {
                                            //if sending the status to client failed
                                            //Store the status in database
                                        }
                                    } else {
                                        //If sending message to recipient is successful
                                        //Send delivered status message to client
                                        let reply_to_client = tx.send(
                                            serde_json::to_string(&MessageStatus::build(
                                                "status".to_string(),
                                                rec_pubkey,
                                                uid.clone(),
                                                "delivered".to_string(),
                                                "true".to_string(),
                                            ))
                                            .unwrap(),
                                        );

                                        if reply_to_client.is_err() {

                                            //Add the Status to DB
                                            //Disconnect Websocket and send server error response to client
                                        }
                                    }
                                } else {
                                    //If user is offline
                                    //Add to database

                                    // let add_to_db = add_message_to_database(
                                    //     &db_client,
                                    //     &message_for_receiver,
                                    //     "Sent",
                                    // )
                                    // .await;

                                    // if add_to_db.is_err() {}
                                    //Send the status of message to client
                                    let reply_to_client = tx.send(
                                        serde_json::to_string(&MessageStatus::build(
                                            "status".to_string(),
                                            rec_pubkey,
                                            uid,
                                            "sent".to_string(),
                                            "true".to_string(),
                                        ))
                                        .unwrap(),
                                    );

                                    if let Err(_) = reply_to_client {

                                        //Store the status in Database
                                    }
                                }
                            }

                            "get_message" => {
                                let unlock_db = db_state;
                                let db_client = unlock_db.read().await;

                                // let get_all_user_messages = sqlx::query(
                                //     "Select * from messages where messageTo=$1 AND status=$2",
                                // )
                                // .bind(&pk)
                                // .bind(&"sent")
                                // .fetch_all(&*db_client)
                                // .await;

                                // if let Err(err) = get_all_user_messages {
                                //     // match err {
                                //     //     sqlx::Error::RowNotFound => {
                                //     //         tx.send(json!({"message_type":"offline_messages","message":"No data","status":false}).to_string())
                                //     //         .unwrap();
                                //     //         continue;
                                //     //     }
                                //     //     _ => {
                                //     //         let sent_to_client = tx.send(
                                //     //             serde_json::to_string(
                                //     //                 r#"{"status":false,"message":"db error"}"#,
                                //     //             )
                                //     //             .unwrap(),
                                //     //         );

                                //     //         if sent_to_client.is_err() {
                                //     //             break;
                                //     //         }

                                //     //         continue;
                                //     //     }
                                //     // }
                                // } else {
                                //     let messages = get_all_user_messages.unwrap();

                                //     if messages.is_empty() {
                                //         tx.send(json!({"message_type":"offline_messages","message":"No data","status":false}).to_string())
                                //          .unwrap();
                                //         continue;
                                //     }

                                //     let mut all_messages = Vec::new();

                                //     for message in messages {
                                //         let message_from: &str = message.get(0);
                                //         let message_to: &str = message.get(1);
                                //         let message_cipher: &str = message.get(2);
                                //         let _status: &str = message.get(3);
                                //         let message_id: &str = message.get(4);
                                //         let time: &str = message.get(5);

                                //         let build_message = RecipientMessage::build(
                                //             "f0".to_string(),
                                //             "offline_messages".to_string(),
                                //             message_cipher.to_string(),
                                //             message_from.to_string(),
                                //             message_to.to_string(),
                                //             message_id.to_string(),
                                //             "wfwds".to_string(),
                                //             time.to_string(),
                                //         );

                                //         all_messages.push(build_message);
                                //     }

                                //     let msgs = serde_json::to_string(&GetMessage {
                                //         message_type: "offline_messages".to_string(),
                                //         messages: all_messages,
                                //         status: true,
                                //     })
                                //     .unwrap();

                                //     let send_to_client = tx.send(msgs);

                                //     if send_to_client.is_err() {
                                //         break;
                                //     }
                                // }
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
            return Err(ClientAuthWsMessage::new(
                "authentication".to_string(),
                false,
                "Invalid JWT".to_string(),
            ));
        }
    } else {
        return Err(ClientAuthWsMessage::new(
            String::from("authentication"),
            false,
            String::from("Invalid JSON Format"),
        ));
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
