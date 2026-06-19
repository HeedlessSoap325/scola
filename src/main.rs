use std::{env, sync::Arc};

use axum::{Json, Router, response::IntoResponse, routing::get};
use dotenv::dotenv;
use serde_json::json;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let port = env::var("BACKEND_PORT").unwrap_or("3000".to_string());
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            println!("Connected to DB successfully");
            pool
        }
        Err(err) => {
            println!("Failed to connect to DB: {}", err);
            std::process::exit(1);
        }
    };

    let app = Router::new()
    .route("/test", get(hello))
    .with_state(Arc::new(AppState { db: pool.clone() }));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Server started successfully at 0.0.0.0:{}", port);

    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> impl IntoResponse {
    let json_response = json!({
        "status": "ok",
        "message": "Hello, World!"
    });
    Json(json_response)
}