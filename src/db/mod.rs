use surrealdb::{Surreal, engine::remote::ws::{Ws, Client}, opt::auth::Root};


pub mod schema;

pub async fn connect_db()->Surreal<Client>{

  // Connect to the server
  let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();
  // Signin as a namespace, database, or root user
  db.signin(Root {
      username: "root",
      password: "root",
  })
  .await.unwrap();

  // Select a specific namespace / database
  db.use_ns("test").use_db("test").await.unwrap();

  db




}