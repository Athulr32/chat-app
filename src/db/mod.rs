use sea_orm::DatabaseConnection;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

pub mod postgres;
pub mod redis_db;
pub mod surreal;

pub async fn connect_db() -> (
    Surreal<Client>,
    DatabaseConnection,
    redis::aio::MultiplexedConnection,
) {
    println!("Connecting to Database");
    // Connect to the server
    let surreal_connection = surreal::establish_db_connection().await;
    println!("Successfully Connected to SurrealDb");

    //Connect to Postgres
    let postgres_connection = postgres::establish_db_connection().await;
    println!("Successfully Connected to Postgres");

    //Connect to Postgres
    let redis = redis_db::establish_db_connection().await;
    println!("Successfully Connected to Redis");

    (surreal_connection, postgres_connection, redis)
}
