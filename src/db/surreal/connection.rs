use surrealdb::{engine::remote::ws::{Client, Ws}, opt::auth::Root, Surreal};



pub async fn establish_db_connection()->Surreal<Client>{

  let db =  Surreal::new::<Ws>("127.0.0.1:8000").await.expect("Failed to Connect to Surreal Db");

        // Signin as a namespace, database, or root user
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .unwrap();
    
        // Select a specific namespace / database
        db.use_ns("test").use_db("test").await.unwrap();
    
        db
}