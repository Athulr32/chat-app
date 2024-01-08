use axum::{
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
// use encryptedapp::register::register;
// use encryptedapp::user_search::user_search;
// use encryptedapp::websocket::ws_handler;
// use encryptedapp::{login::login, updateStatus::update_status_of_message};
use chatserver::{types::AppState, db, auth::{register::{self, register}, self}};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
// use encryptedapp::get_message::get_message;

//-> shuttle_axum::ShuttleAxum
#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_client = db::connect_db().await;

    //CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    //Map Which will store the user Id that is public Key and the users channel variable as value
    let state: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>> =
        Arc::new(RwLock::new(HashMap::new()));


    let pool = Arc::new(RwLock::new(db_client));

    let app_state: Arc<AppState> = Arc::new(AppState::new(state, pool));

    //3. APP Router
    let app = Router::new()
        .merge(auth::router(app_state.clone()))
        // .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(app_state.clone());

    // .route("/getMessage", get(get_message))
    // .route("/register", post(register))
    // .route("/userSearch", post(user_search))
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
