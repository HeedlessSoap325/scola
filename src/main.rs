use axum::{Router, routing::{get, post}};
use dotenv::dotenv;
use tower_cookies::CookieManagerLayer;

mod common;
mod routes;

use crate::{common::state::AppState, routes::auth::handlers::{login, logout, me}};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let port = std::env::var("BACKEND_PORT")
		.unwrap_or("3000".to_string());

    let state = AppState::new().await;

    let app = Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .with_state(state)
        .layer(CookieManagerLayer::new());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Server started successfully at 0.0.0.0:{}", port);

    axum::serve(listener, app).await.unwrap();
}