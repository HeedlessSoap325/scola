use axum::{Router, routing::{delete, get, patch, post}};
use dotenv::dotenv;
use tower_cookies::CookieManagerLayer;

mod common;
mod routes;

use crate::{common::state::AppState, routes::{auth::handlers::{login, logout, logout_all, me}, class::handlers::{add_class, delete_class, edit_class, get_classes}, course::handlers::{add_course, delete_course, edit_course, get_courses}, school::handlers::{add_school, get_schools}}};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let port = std::env::var("BACKEND_PORT")
		.unwrap_or("3000".to_string());

    let state = AppState::new().await;

    let app = Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/logout_all", post(logout_all))
        .route("/auth/me", get(me))

        .route("/class", get(get_classes))
        .route("/class", post(add_class))
        .route("/class/{class_id}", delete(delete_class))
        .route("/class/{class_id}", patch(edit_class))

        .route("/course", get(get_courses))
        .route("/course", post(add_course))
        .route("/course/{course_id}", delete(delete_course))
        .route("/course/{course_id}", patch(edit_course))
        
        .route("/school", get(get_schools))
        .route("/school", post(add_school))
        .with_state(state)
        .layer(CookieManagerLayer::new());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Server started successfully at 0.0.0.0:{}", port);

    axum::serve(listener, app).await.unwrap();
}