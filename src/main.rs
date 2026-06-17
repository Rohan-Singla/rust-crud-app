use std::env;

use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::{get, post}};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, pool, postgres::PgPoolOptions, prelude::FromRow , PgPool};
use tokio::net::TcpListener;


// serialize and deserialize macro so serde functions can be used
#[derive(Serialize,Deserialize)]
struct UserPayload {
    name : String,
    email : String
}

// From row is required so rust knows what will be the recieving data basically table structure
#[derive(Serialize,FromRow)]
struct User {
    id : i32,
    name : String,
    email : String
}
#[tokio::main]
async fn main() {
    // get the database url from the enviournment docker in this case

    let db_url = env::var("DATABASE_URL").expect("DB Url does not exist !!");

    // make a connection to DB
    let pool = PgPoolOptions::new().connect(&db_url).await.expect("Failde to connect to DB");

    // runs all migrations under migrations folder on start

    sqlx::migrate!().run(&pool).await.expect("Migrations Failed");


    // axum's api design is calling different http methods with different handlers so no need to repeat handlers

    let app = Router::new()
    .route("/", get(root))
    .route("/users", post(create_user).get(list_users))
    .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
    .with_state(pool)
    ;

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Server running on port 8000");

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str{
    "Welcome to the user management API"
}

// /State(pool)     ← destructure: extract the inner value and name it `pool`
 // State<PgPool>   ← the type: axum's State wrapper containing a PgPool
 // Returns a Vec of user and serializes into Json

async fn list_users (State(pool) : State<PgPool>) -> Result<Json<Vec<User>>,StatusCode>{

    return sqlx::query_as::<_,User>("Select * from users")
    .fetch_all(&pool).await.map(Json).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR);

}

async fn create_user (State(pool) : State<PgPool>, Json(payload) : Json<UserPayload>) -> Result<(StatusCode,Json<User>),StatusCode>{
     
     
        return sqlx::query_as::<_,User>("INSERT INTO users (name,email) VALUES ($1,$2) RETURNING *")
        .bind(payload.name)
        .bind(payload.email)
        .fetch_one(&pool).await
        .map(|u|(StatusCode::CREATED,Json(u)))
        .map_err(|_|StatusCode::INTERNAL_SERVER_ERROR);


}

async fn get_user(
    State(pool): State<PgPool>,
    Path(id): Path<i32>
    ) -> Result<Json<User>, StatusCode> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool).await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<UserPayload>
    ) -> Result<Json<User>, StatusCode> {
    sqlx::query_as::<_, User>("UPDATE users SET name = $1, email = $2 WHERE id = $3 RETURNING *")
        .bind(payload.name)
        .bind(payload.email)
        .bind(id)
        .fetch_one(&pool).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<i32>
) -> Result<StatusCode, StatusCode> {
    let result = sqlx
        ::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}