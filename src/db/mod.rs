use sea_orm::DatabaseConnection;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

pub mod surreal;
pub mod postgres;


pub async fn connect_db() -> (Surreal<Client>,DatabaseConnection) {
    println!("Connecting to Database");
    // Connect to the server
    let surreal_connection = surreal::establish_db_connection().await;
    println!("Successfully Connected to SurrealDb");

    //Connect to Postgres
    let postgres_connection = postgres::establish_db_connection().await;
    println!("Successfully Connected to Postgres");

    (surreal_connection,postgres_connection)
}
