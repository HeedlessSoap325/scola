use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow, Default)]
pub struct GetTeacherResponse {
	pub id: Uuid,
	pub school_id: Uuid,
	pub email: String,
	pub first_name: String,
	pub last_name: String,
	pub picture: Option<String>,
	pub address: String,
	pub abbreviation: String,
	pub phone: String,
}

#[derive(Deserialize)]
pub struct CreateTeacherRequest {
	pub school_id: Option<Uuid>,
	pub email: String,
	pub login_name: String,
	pub first_name: String,
	pub last_name: String,
	pub picture: Option<String>,
	pub password: String,
	pub address: String,
	pub abbreviation: String,
	pub phone: String,
}

#[derive(Deserialize)]
pub struct PatchTeacherRequest {
	pub school_id: Option<Uuid>,
	pub email: Option<String>,
	pub login_name: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
	pub picture: Option<String>,
	pub password: Option<String>,
	pub address: Option<String>,
	pub abbreviation: Option<String>,
	pub phone: Option<String>,
}