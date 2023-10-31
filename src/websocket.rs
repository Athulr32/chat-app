use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use sqlx::{postgres::PgQueryResult, Pool, Postgres, Row};

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::{broadcast, RwLock};

use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;

use crate::types::{
    AppState, ClientAuthWsMessage, ClientMessage, ClientWsMessageInvalidJsonFormat, GetMessage,
    MessageStatus, RecipientMessage, SocketAuthUserMessage,
};
use sha2::Sha256;
use std::collections::BTreeMap;

#[axum_macros::debug_handler]
pub async fn ws_handler(ws: WebSocketUpgrade, State(app_state): State<Arc<AppState>>) -> Response {
    //upgrade the websocket connection
    ws.on_failed_upgrade(|_| {})
        .on_upgrade(move |socket| handle_socket(socket, app_state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
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
        let name = String::from("");

        //Check for message from the client in receiever socket instance
        while let Some(Ok(socket_message)) = receiver.next().await {
            match socket_message {
                Message::Text(msg) => {
                    //Client authentication to Web Socket
                    if !auth {
                        //Verify and decode Client JWT TOKEN
                        let decode_socket_auth: Result<SocketAuthUserMessage, serde_json::Error> =
                            serde_json::from_str(&msg);

                        if let Ok(auth_details) = decode_socket_auth {
                            //Check if details are correct
                            //If yes Add to authenticated pool
                            //Add the public key and channel

                            let token = auth_details.get_token();

                            let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

                            let claims: Result<BTreeMap<String, String>, jwt::Error> =
                                token.verify_with_key(&key);

                            //BLOCK TO GET DETAILS FROM JWT
                            if let Ok(claim) = claims {
                                pk = claim["public_key"].to_string();

                                let db = state.get_db_client();
                                let db_client = db.read().await;

                                let username =
                                    sqlx::query("SELECT name from USERS where publicKey=$1")
                                        .bind(&pk)
                                        .fetch_one(&*db_client)
                                        .await;

                                if let Ok(user) = username {
                                    if user.is_empty() {
                                        break;
                                    }

                                    let _name: String = user.get(0);
                                }

                                let app_state_arc = state.get_state();
                                let mut app_state = app_state_arc.write().await;
                                //Check if user is already logged in
                                //If yes discard the socket

                                let user_connection_exist = app_state.get(&pk);

                                if user_connection_exist.is_none() {
                                    app_state.insert(pk.to_string(), tx.clone());

                                    auth = true;

                                    let reply_message = ClientAuthWsMessage::new(
                                        "authentication".to_string(),
                                        true,
                                        "user authenticated".to_string(),
                                    );

                                    let json_convert =
                                        serde_json::to_string(&reply_message).unwrap();

                                    let reply_to_client = tx.send(json_convert);

                                    if let Err(_) = reply_to_client {
                                        break;
                                    }
                                } else {
                                    let reply_message = ClientAuthWsMessage::new(
                                        "authentication".to_string(),
                                        false,
                                        "user already logged in".to_string(),
                                    );

                                    let json_convert =
                                        serde_json::to_string(&reply_message).unwrap();

                                    let reply_to_client = tx.send(json_convert);

                                    if let Err(_) = reply_to_client {
                                        break;
                                    }
                                }
                            }
                            //BLOCK TO EXECUTE IF JWT IS INVALID
                            else {
                                let reply_message = ClientAuthWsMessage::new(
                                    "authentication".to_string(),
                                    false,
                                    "Invalid JWT".to_string(),
                                );

                                let json_convert = serde_json::to_string(&reply_message).unwrap();

                                let reply_to_client = tx.send(json_convert);

                                if let Err(_) = reply_to_client {
                                    break;
                                }
                            }
                        }
                        //Client sent invalid JSON format for websocket authentication
                        else {
                            let reply_message = ClientAuthWsMessage::new(
                                "authentication".to_string(),
                                false,
                                "Invalid JSON format".to_string(),
                            );

                            let json_convert = serde_json::to_string(&reply_message).unwrap();

                            let reply_to_client = tx.send(json_convert);

                            if let Err(_) = reply_to_client {
                                break;
                            }
                        }
                    } else {
                        //If user is authenticated Execute this Block

                        //Client message
                        let get_msg: Result<serde_json::Value, serde_json::Error> =
                            serde_json::from_str(&msg);

                        //if the message is not in proper json format send error message to client
                        if get_msg.is_err() {
                            //Client sent invalid format
                            let convert_to_json =
                                serde_json::to_string(&ClientWsMessageInvalidJsonFormat::default())
                                    .unwrap();

                            let send_to_client = tx.send(convert_to_json);

                            if let Err(_) = send_to_client {
                                break;
                            }

                            continue;
                        }

                        //WANT message_type field in the JSON if not avaialble send error message
                        let client_message: serde_json::Value = get_msg.unwrap();

                        let message_type = client_message.get("message_type");

                        if message_type.is_none() {
                            let convert_to_json =
                                serde_json::to_string(&ClientWsMessageInvalidJsonFormat::build())
                                    .unwrap();

                            let send_to_user = tx.send(convert_to_json);
                            if let Err(_) = send_to_user {
                                break;
                            }
                            continue;
                        }

                        let message_type = message_type.unwrap().as_str();

                        if message_type.is_none() {
                            let convert_to_json =
                                serde_json::to_string(&ClientWsMessageInvalidJsonFormat::build())
                                    .unwrap();

                            let send_to_user = tx.send(convert_to_json);
                            if let Err(_) = send_to_user {
                                break;
                            }
                            continue;
                        }

                        //Now  there is Message_type field in JSON sent by the client
                        let message_type = message_type.unwrap();

                        match message_type {
                            "private_message" => {
                                //WANT CLIENT MESSAGE IN THIS FORMAT ONLY FOR PRIVATE MESSAGE
                                let user_message: Result<ClientMessage, serde_json::Error> =
                                    serde_json::from_str(&msg);

                                if user_message.is_err() {
                                    let convert_to_json = serde_json::to_string(
                                        &ClientWsMessageInvalidJsonFormat::default(),
                                    )
                                    .unwrap();

                                    let send_to_client = tx.send(convert_to_json);

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
                                let unlock_state = state.get_state();

                                let unlock_state = unlock_state.read().await;

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
                                let db_client = state.get_db_client();

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
                                        let add_to_db = add_message_to_database(
                                            &db_client,
                                            &message_for_receiver,
                                            "Sent",
                                        )
                                        .await;

                                        if add_to_db.is_err() {

                                            //Disconnect Websocket and send server error response to client
                                        }

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

                                    let add_to_db = add_message_to_database(
                                        &db_client,
                                        &message_for_receiver,
                                        "Sent",
                                    )
                                    .await;

                                    if add_to_db.is_err() {}
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
                                let unlock_db = state.get_db_client();
                                let db_client = unlock_db.read().await;

                                let get_all_user_messages = sqlx::query(
                                    "Select * from messages where messageTo=$1 AND status=$2",
                                )
                                .bind(&pk)
                                .bind(&"sent")
                                .fetch_all(&*db_client)
                                .await;

                                if let Err(err) = get_all_user_messages {
                                    match err {
                                        sqlx::Error::RowNotFound => {
                                            tx.send(json!({"message_type":"offline_messages","message":"No data","status":false}).to_string())
                                            .unwrap();
                                            continue;
                                        }
                                        _ => {
                                            let sent_to_client = tx.send(
                                                serde_json::to_string(
                                                    r#"{"status":false,"message":"db error"}"#,
                                                )
                                                .unwrap(),
                                            );

                                            if sent_to_client.is_err() {
                                                break;
                                            }

                                            continue;
                                        }
                                    }
                                } else {
                                    let messages = get_all_user_messages.unwrap();

                                    if messages.is_empty() {
                                        tx.send(json!({"message_type":"offline_messages","message":"No data","status":false}).to_string())
                                         .unwrap();
                                        continue;
                                    }

                                    let mut all_messages = Vec::new();

                                    for message in messages {
                                        let message_from: &str = message.get(0);
                                        let message_to: &str = message.get(1);
                                        let message_cipher: &str = message.get(2);
                                        let _status: &str = message.get(3);
                                        let message_id: &str = message.get(4);
                                        let time: &str = message.get(5);

                                        let build_message = RecipientMessage::build(
                                            "f0".to_string(),
                                            "offline_messages".to_string(),
                                            message_cipher.to_string(),
                                            message_from.to_string(),
                                            message_to.to_string(),
                                            message_id.to_string(),
                                            "wfwds".to_string(),
                                            time.to_string(),
                                        );

                                        all_messages.push(build_message);
                                    }

                                    let msgs = serde_json::to_string(&GetMessage {
                                        message_type: "offline_messages".to_string(),
                                        messages: all_messages,
                                        status: true,
                                    })
                                    .unwrap();

                                    let send_to_client = tx.send(msgs);

                                    if send_to_client.is_err() {
                                        break;
                                    }
                                }
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

        let unlock_state = state.get_state();
        let mut unlock_state = unlock_state.write().await;

        unlock_state.remove(&pk[..]);

        receiver_handler.abort();

        println!("Disconnected");
    });
}

pub async fn add_message_to_database(
    db_client: &Arc<RwLock<Pool<Postgres>>>,
    message: &RecipientMessage,
    status: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    let pool = db_client.write().await;

    sqlx::query("INSERT INTO MESSAGES VALUES($1,$2,$3,$4,$5,$6)")
        .bind(message.get_message_from())
        .bind(message.get_message_to())
        .bind(message.get_cipher())
        .bind(status)
        .bind(message.get_message_id())
        .bind(message.get_time())
        .execute(&*pool)
        .await
}
