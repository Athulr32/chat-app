use std::env;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use std::thread;

use dotenvy::{dotenv};

pub async fn establish_db_connection() -> DatabaseConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db: DatabaseConnection = Database::connect(database_url).await.expect("Failed to Connect to Database");
    db

}