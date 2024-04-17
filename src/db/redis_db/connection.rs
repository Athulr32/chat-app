pub async fn establish_db_connection() -> redis::aio::MultiplexedConnection {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to Connect to Redis");
    let mut con = client
        .get_multiplexed_tokio_connection()
        .await
        .expect("Failed to Connect to Redis");

    return con;
}
