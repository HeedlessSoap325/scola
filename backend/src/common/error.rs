use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use deadpool_redis::PoolError;
use redis::RedisError;

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

/// Prints a readable error message to STDOUT and sends the status "Internal Server Error" to the client
pub fn db_error(e: sqlx::Error) -> AppError {
    println!("[SQLX] {}", sqlx_error_message(e));
    AppError(StatusCode::INTERNAL_SERVER_ERROR, "Database Query failed")
}

fn sqlx_error_message(err: sqlx::Error) -> String {
    match err {
        sqlx::Error::Database(db_error) => {
            format!("Database Error: {}", db_error.message()) // general database error, like missing commas, to many commas, etc.
        }
        sqlx::Error::Configuration(error) => {
            format!("Error while parsing connection String: {}", error)
        },
        sqlx::Error::InvalidArgument(error) => {
            format!("Invalid argument(s) to called function: {}", error)
        },
        sqlx::Error::Io(error) => {
            format!("Error on communication with database backend: {}", error)
        },
        sqlx::Error::Tls(error) => {
            format!("Error while trying to establish a TLS connection: {}", error)
        },
        sqlx::Error::Protocol(error) => {
            format!("Unexpected or invalid data received while communicating with Database: {}", error) // well, I guess this is the end
        },
        sqlx::Error::RowNotFound => "Query was expected to return at least one Row but returned none".to_string(),
        sqlx::Error::TypeNotFound { type_name } => {
            format!("Unknown type in query: {}", type_name)
        },
        sqlx::Error::ColumnIndexOutOfBounds { index, len } => {
            format!("Index of column out of bounds! index: {}, len: {}", index, len)
        },
        sqlx::Error::ColumnNotFound(col) => {
            format!("No column with name {} was found", col)
        },
        sqlx::Error::ColumnDecode { index, source } => {
            format!("Error while decoding value at index {}: {}", index, source)
        },
        sqlx::Error::Encode(error) => {
            format!("Error while encoding a value: {}", error)
        },
        sqlx::Error::Decode(error) => {
            format!("Error while decoding a value: {}", error)
        },
        sqlx::Error::AnyDriverError(error) => {
            format!("Error occurred within the Any driver mapping to/from the native driver: {}", error) // hopefully won't happen
        },
        sqlx::Error::PoolTimedOut => "Database connection Pool timed out".to_string(),
        sqlx::Error::PoolClosed => "Database connection Pool closed".to_string(),
        sqlx::Error::WorkerCrashed => "A backgroun worker has crashed".to_string(), // probably won't happen
        sqlx::Error::Migrate(migrate_error) => {
            format!("Error while migrating the schema: {}", migrate_error)
        }, // shouldn't happen, as we don't migrate in this backend
        sqlx::Error::InvalidSavePointStatement => "Invalid save point statement".to_string(), // probably won't happen
        sqlx::Error::BeginFailed => "Begin failed".to_string(), // probably won't happen
        _ => {
            format!("Query failed: {}", err)
        },
    }
}

pub fn redis_pool_error(e: PoolError) -> AppError {
    println!("[Redis] {}", e.to_string());  
    AppError(StatusCode::INTERNAL_SERVER_ERROR, "Pool error")
}

pub fn redis_error(e: RedisError) -> AppError {
    println!("[Redis] {}", e.to_string());  
    AppError(StatusCode::INTERNAL_SERVER_ERROR, "Redis error")
}
