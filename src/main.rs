use std::env;

use axum::{Json, Router, response::IntoResponse, routing::get};
use serde_json::json;

#[tokio::main]
async fn main() {
    let port = env::var("BACKEND_PORT").unwrap_or("3000".to_string());

    let app = Router::new()
    .route("/test", get(hello));

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