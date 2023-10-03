use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    headers::{self, authorization::Bearer},
    response::Response,
    Extension, TypedHeader,
};
use futures_util::{lock::Mutex, SinkExt, StreamExt};

use serde_json::json;

use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, RwLock};

use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;

use crate::types::{
    AppState, ClientMessage, GetMessage, MessageStatus, RecieverMessage, SocketAuth,
};
use sha2::Sha256;
use std::collections::BTreeMap;

#[axum_macros::debug_handler]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<AppState>>,
) -> Response {
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
    let sender_handler = tokio::spawn(async move {
        let mut auth = false;
        let mut pk = String::from("");
        let mut name = String::from("");

        //Check for message from the user in receiever socket instance
        while let Some(Ok(socket_message)) = receiver.next().await {
            match socket_message {
            Message::Text(msg) => {
                //User authentication to Web Socket
                if !auth {

                    let decode_socket_auth: Result<SocketAuth, serde_json::Error> =
                        serde_json::from_str(&msg);
             
                    if let Ok(auth_details) = decode_socket_auth {
                        //Check if details are correct
                        //If yes Add to authenticated pool
                        //Add the public key and channel

                        let token = auth_details.get_token();

                        let key: Hmac<Sha256> = Hmac::new_from_slice(b"abcd").unwrap();

                        let claims: Result<BTreeMap<String, String>, jwt::Error> =
                            token.verify_with_key(&key);

                        if let Ok(claim) = claims {
                            pk = claim["public_key"].to_string();

                            let db = state.get_db_client();
                            let db_client = db.read().await;

                            //get the name of user
                            let username = db_client
                                .query("SELECT name from USERS where publicKey=$1", &[&pk])
                                .await
                                .unwrap();

                            if !username.is_empty() {
                                let user_name: &str = username[0].get(0);
                                println!("{}", user_name);

                                name = user_name.to_string();
                            } else {
                                break;
                            }

                            let mut muttex = state.lock().await;
                            //Check if user is already logged in
                            //If yes discard the socket

                            let user_connection_exist = muttex.get(&pk);

                            if user_connection_exist.is_none() {
                                muttex.insert(pk.to_string(), tx.clone());

                                println!("{:?}", muttex);
                                auth = true;
                                //Check if the user didn't recieve any previous message
                                //If not give info in the JSOn
                                // will be done at last
                                tx.send(
                                    r#"{
                                        "message_type":"authentication",
                                        "status":"true",
                                        "message":"user authenticated"
                                }"#
                                    .to_string(),
                                )
                                .unwrap();
                            } else {
                                tx.send(
                                    r#"{
                                        "message_type":"authentication",
                                        "status":"false",
                                        "message":"user already logged in"
                                }"#
                                    .to_string(),
                                )
                                .unwrap();

                                continue;
                            }
                        } else {
                            tx.send(
                                r#"{
                                    "message_type":"authentication",
                                    "status":"false",
                                    "message":"invalid key"
                                }"#
                                .to_string(),
                            )
                            .unwrap();

                            continue;
                        }
                    } else {
                        //Client sent invalid format
                        tx.send(
                            r#"{
                                "type":"authentication",
                                "status":"false",
                                "message":"invalid format"
                             }"#
                            .to_string(),
                        )
                        .unwrap();

                        continue;
                    }
                } else {
                    //User message
                    let get_msg: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_str(&msg);

                    if get_msg.is_err() {
                        //Client sent invalid format
                        tx.send(
                            r#"{
                                "type":"message_format",
                                "status":"false",
                                "message":"send valid json"
                             }"#
                            .to_string(),
                        )
                        .unwrap();

                        continue;
                    }

                    let get_msg: serde_json::Value = serde_json::from_str(&msg).unwrap();

                    let message_type = get_msg.get("message_type");

                    if message_type.is_none() {
                        tx.send(
                            r#"{
                                "type":"message_format",
                                "status":"false",
                                "message":"send valid json"
                             }"#
                            .to_string(),
                        )
                        .unwrap();

                        continue;
                    }

                    let message_type = get_msg.get("message_type").unwrap().as_str();

                    if message_type.is_none() {
                        tx.send(
                            r#"{
                                "type":"message_format",
                                "status":"false",
                                "message":"send valid json"
                             }"#
                            .to_string(),
                        )
                        .unwrap();

                        continue;
                    }

                    let message_type = get_msg.get("message_type").unwrap().as_str().unwrap();
                    match message_type {
                        "private_message" => {
                            let user_message: ClientMessage =
                                serde_json::from_str(&msg).unwrap();

                            //Reciever public key
                            let rec_pubkey: String = user_message.get_public_key();

                            let uid: String = user_message.get_uid();

                            let unlock_state = state.lock().await;

                            //Check whether reciever is online
                            let tr = unlock_state.get(&rec_pubkey);
                            let time = SystemTime::now();
                            let since_the_epoch = time
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards");
                            let current_time = since_the_epoch.as_secs() * 1000
                                + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

                            println!("{}", current_time);
                            //Construct message for the receiver also to add in DB
                            let message_for_receiver = RecieverMessage::build(
                                user_message.get_uid(),
                                user_message.message_type.clone(),
                                user_message.get_cipher(),
                                pk.clone(),
                                "fd".to_string(),
                                name.to_string(),
                                current_time.to_string(),
                            );

                            if let Some(tr) = tr {
                                //If user is online
                                //Send message to user

                                let send_message = tr.send(
                                    serde_json::to_string(&message_for_receiver).unwrap(),
                                );

                                if send_message.is_err() {
                                    //If sending messages to reciever failed
                                    //Add to database
                                    let _insert_msg = add_message_to_database(
                                        &client,
                                        &pk,
                                        &user_message,
                                        &uid,
                                        &"sent",
                                        &current_time.to_string(),
                                    )
                                    .await;

                                    let _ = tx.send(
                                        serde_json::to_string(&MessageStatus::build(
                                            "status".to_string(),
                                            rec_pubkey,
                                            uid,
                                            "sent".to_string(),
                                            "true".to_string(),
                                        ))
                                        .unwrap(),
                                    );
                                } else {
                                    //If sending message to reciever is successful

                                    tx.send(
                                        serde_json::to_string(&MessageStatus::build(
                                            "status".to_string(),
                                            rec_pubkey,
                                            uid.clone(),
                                            "delivered".to_string(),
                                            "true".to_string(),
                                        ))
                                        .unwrap(),
                                    )
                                    .unwrap();

                                    let _insert_msg = add_message_to_database(
                                        &client,
                                        &pk,
                                        &user_message,
                                        &uid,
                                        &"delivered",
                                        &current_time.to_string(),
                                    )
                                    .await;
                                }
                            } else {
                                //If user is offline
                                //Add to database

                                let _insert_msg = add_message_to_database(
                                    &client,
                                    &pk,
                                    &user_message,
                                    &uid,
                                    &"sent",
                                    &current_time.to_string(),
                                )
                                .await
                                .unwrap();
                                println!("{:?}", _insert_msg);
                                tx.send(
                                    serde_json::to_string(&MessageStatus::build(
                                        "status".to_string(),
                                        rec_pubkey,
                                        uid,
                                        "sent".to_string(),
                                        "true".to_string(),
                                    ))
                                    .unwrap(),
                                )
                                .unwrap();
                            }
                        }
                        "get_message" => {
                            println!("Got");
                            let unlock_db = client.read().await;
                            let get_all_user_messages = unlock_db
                                .query(
                                    "SELECT * from messages where messageTo=$1 AND status=$2 ",
                                    &[&pk, &"sent"],
                                )
                                .await;

                            if get_all_user_messages.is_err() {
                                tx.send(
                                    serde_json::to_string(
                                        r#"{"status":false,"message":"db error"}"#,
                                    )
                                    .unwrap(),
                                )
                                .unwrap();
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
                                    let status: &str = message.get(3);
                                    let message_id: &str = message.get(4);
                                    let time: &str = message.get(5);

                                    let build_message = RecieverMessage::build(
                                        "f0".to_string(),
                                        "offline_messages".to_string(),
                                        message_cipher.to_string(),
                                        message_from.to_string(),
                                        message_id.to_string(),
                                        "wfwds".to_string(),
                                        time.to_string(),
                                    );

                                    all_messages.push(build_message);
                                }

                                tx.send(
                                    serde_json::to_string(&GetMessage {
                                        message_type: "offline_messages".to_string(),
                                        messages: all_messages,
                                        status: true,
                                    })
                                    .unwrap(),
                                )
                                .unwrap();
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

        let mut unlock_state = state.get_state();

        // unlock_state.remove(&pk[..]);
        // handler.abort();
        println!("Disconnected");
    });
}

// pub async fn add_message_to_database(
//     client: &Arc<RwLock<Client>>,
//     pk: &str,
//     user_message: &ClientMessage,
//     message_id: &str,
//     status: &str,
//     time: &str,
// ) -> Result<u64, Error> {
//     let unlock_client = client.read().await;

//     println!("{:?}", status);
//     unlock_client
//         .execute(
//             "INSERT INTO MESSAGES VALUES($1,$2,$3,$4,$5,$6)",
//             &[
//                 &pk,
//                 &user_message.get_public_key(),
//                 &user_message.get_cipher(),
//                 &status,
//                 &message_id,
//                 &time,
//             ],
//         )
//         .await
// }

//messageFrom TEXT,messageTo TEXT,message TEXT,status TEXT,messageId TEXT,timestamp TEXT
