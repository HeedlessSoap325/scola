use axum::{Router, routing::{delete, get, patch, post}};
use dotenv::dotenv;
use tower_cookies::CookieManagerLayer;

mod common;
mod routes;

use crate::{common::state::AppState, routes::{auth::handlers::{login, logout, logout_all, me}, class::handlers::{add_class, delete_class, edit_class, get_classes}, course::handlers::{add_course, delete_course, edit_course, get_courses}, grade::handlers::get_grades, room::handlers::{add_room, delete_room, edit_room, get_rooms}, school::handlers::{add_school, delete_school, edit_school, get_schools}, semester::handlers::{add_semester, delete_semester, edit_semester, get_semesters}, timetable::handlers::get_timetable}};

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
        .route("/school/{school_id}", patch(edit_school))
        .route("/school/{school_id}", delete(delete_school))

        .route("/semester", get(get_semesters))
        .route("/semester", post(add_semester))
        .route("/semester/{semester_id}", patch(edit_semester))
        .route("/semester/{semester_id}", delete(delete_semester))

        .route("/room", get(get_rooms))
        .route("/room", post(add_room))
        .route("/room/{room_id}", patch(edit_room))
        .route("/room/{room_id}", delete(delete_room))

        .route("/grade", get(get_grades))

        .route("/timetable", get(get_timetable))
        .with_state(state)
        .layer(CookieManagerLayer::new());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Server started successfully at 0.0.0.0:{}", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install ctrl+C handler");
    println!("Stopping Server")
}