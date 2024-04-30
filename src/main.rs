use ::blockchain::token_balance::fetch_eth_balance;
use axum::{
    routing::{get, post},
    Router,
};
use chatserver::{
    api::{
        auth::{
            self,
            register::{self, register},
        },
        blockchain,
        get_message::get_message,
        get_message_on_bootstrap, send_message,
        types::AppState,
        typing::typing,
        user_search::user_search,
        websocket::ws_handler,
    },
    db,
};
use dotenvy::dotenv;
use futures_util::lock::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
// use encryptedapp::get_message::get_message;

//-> shuttle_axum::ShuttleAxum
#[tokio::main]
async fn main() {
    dotenv().ok();

    let (surreal_connection, postgres_connection, redis) = db::connect_db().await;

    // fetch_eth_balance(postgres_connection.clone(), redis.clone()).await;
    //CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    //Map Which will store the user Id that is public Key and the users channel variable as value
    let state: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let surreal_db_connection = Arc::new(RwLock::new(surreal_connection));
    let postgres_db_connection = Arc::new(RwLock::new(postgres_connection));
    let redis_db_connection = Arc::new(RwLock::new(redis.clone()));
    let app_state: Arc<RwLock<AppState>> = Arc::new(RwLock::new(AppState::new(
        state,
        surreal_db_connection,
        postgres_db_connection,
        redis_db_connection,
    )));

    //3. APP Router
    let app = Router::new()
        .merge(auth::router(app_state.clone()))
        .route("/ws", get(ws_handler))
        .route("/userSearch", post(user_search))
        .nest("/blockchain", blockchain::router(app_state.clone()))
        .route("/sendMessage", post(send_message::send_message))
        .route("/user/:userId/messages", get(get_message))
        .route(
            "/messages/getMessagesOnBootstrap",
            get(get_message_on_bootstrap::get_message_on_boostrap),
        )
        .route("/typing/:userId", get(typing))
        .layer(cors)
        .with_state(app_state.clone());
    // .route("/getMessage", get(get_message))

    // .route("/updateStatus", post(update_status_of_message))
    // .layer(Extension(state))
    // .layer(cors)
    // .with_state(new_client.clone());

    //4. Start the Axum Server
    axum::Server::bind(&"127.0.0.1:3011".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
