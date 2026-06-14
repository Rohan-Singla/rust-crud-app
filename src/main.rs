use std::env;

use axum::{Router, routing::{get, post}};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, prelude::FromRow};
use tokio::net::TcpListener;


#[derive(Serialize,Deserialize)]
struct UserPayload {
    name : String,
    email : String
}

#[derive(Serialize,FromRow)]
struct User {
    id : i32,
    name : String,
    email : String
}
#[tokio::main]
async fn main() {
    let db_url = env::var("DATABASE_URL").expect("DB Url does not exist !!");

    let pool = PgPoolOptions::new().connect(&db_url).await.expect("Failde to connect to DB");

    sqlx::migrate!().run(&pool).await.expect("Migrations Failed");

    let app = Router::new()
    .route("/", get(root))
    .route("/users", post(create_user).get(list_users))
    .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
    .with_state(pool)
    ;

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Server running on port 8000");

    axum::serve(listener, app);
}

async fn root() -> &'static str{
    "Welcome to the user management API"
}