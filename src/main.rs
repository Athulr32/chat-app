use axum::{
    routing::{get, post, Route},
    Extension, Router,
};
use dotenvy::dotenv;
// use encryptedapp::register::register;
// use encryptedapp::user_search::user_search;
// use encryptedapp::websocket::ws_handler;
// use encryptedapp::{login::login, updateStatus::update_status_of_message};
use chatserver::types::AppState;
use chatserver::{login::login, websocket::ws_handler};
use futures_util::lock::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
// use encryptedapp::get_message::get_message;
use sqlx::{pool, sqlite::SqlitePool};

//-> shuttle_axum::ShuttleAxum
#[tokio::main]
async fn main() {
    dotenv().ok();
    //CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    //Map Which will store the user Id that is public Key and the users channel variable as value
    let state: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    //DB connection
    //Share the Pool Connection Across the endpoints
    let pool = Arc::new(RwLock::new(
        SqlitePool::connect("sqlite:encryptedapp.db").await.unwrap(),
    ));

    let app_state: Arc<AppState> = Arc::new(AppState::new(state, pool.clone()));

    //Creating USER table
    let pool_conn = pool.clone();
    let conn = pool_conn.write().await;
    let _create_user_table = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS USERS(name TEXT UNIQUE ,publicKey TEXT PRIMARY KEY )",
    )
    .execute(&*conn)
    .await
    .unwrap();

    //Creating Message Table
    let _create_message_table = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS MESSAGES(messageFrom TEXT,messageTo TEXT,message TEXT,status TEXT,messageId TEXT,timestamp TEXT,FOREIGN KEY(messageFrom) REFERENCES USERS(publicKey))",
    )
    .execute(&*conn)
    .await
    .unwrap();

    drop(conn);

    //APP Router
    let app = Router::new()
        .route("/login", post(login))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(app_state);

    // .route("/getMessage", get(get_message))
    // .route("/register", post(register))
    // .route("/userSearch", post(user_search))
    // .route("/updateStatus", post(update_status_of_message))
    // .layer(Extension(state))
    // .layer(cors)
    // .with_state(new_client.clone());

    //Axum Server
    axum::Server::bind(&"127.0.0.1:3011".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    println!("Hi");
}