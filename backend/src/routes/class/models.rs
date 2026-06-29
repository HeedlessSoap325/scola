use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow)]
pub struct GetClassStudent {
	pub first_name: String,
	pub last_name: String,
	pub email: String
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct GetClassResponse {
    pub class_name: String,
    pub class_abbreviation: String,
    pub teacher_first_name: String,
    pub teacher_last_name: String,
    pub teacher_email: String,
    pub persons: sqlx::types::Json<Vec<GetClassStudent>>,
}

#[derive(Deserialize)]
pub struct CreateClassRequest {
	pub name: String,
	pub abbreviation: String,
    pub description: String,
	pub teacher: Uuid,
	pub school_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct PatchClassRequest {
	pub name: Option<String>,
	pub abbreviation: Option<String>,
    pub description: Option<String>,
	pub teacher: Option<Uuid>,
}