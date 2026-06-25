use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};

pub struct AppError(
	pub StatusCode, 
	pub &'static str,
);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({ "error": self.1 }));
        (self.0, body).into_response()
    }
}

pub fn db_error(_: sqlx::Error) -> AppError {
    AppError(StatusCode::INTERNAL_SERVER_ERROR, "Database Query failed")
}