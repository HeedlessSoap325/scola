use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GetCourseRequest {
	pub semester_id: Uuid,
}

#[derive(Serialize, FromRow)]
pub struct GetCourseResponse {
	pub course_id: Uuid,
    pub course_name: String,
    pub course_abbreviation: String,
    pub course_description: String,
	pub teacher_first_name: String,
    pub teacher_last_name: String,
    pub teacher_email: String,
    pub teacher_phone: String,
    pub teacher_address: String,
	pub class_name: String,
}

#[derive(Deserialize)]
pub struct CreateCourseRequest {
    pub name: String,
    pub abbreviation: String,
    pub description: String,
	pub teacher: Uuid,
	pub class: Uuid,
	pub semester: Uuid,
	pub school_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct PatchCourseRequest {
    pub name: Option<String>,
    pub abbreviation: Option<String>,
    pub description: Option<String>,
	pub teacher: Option<Uuid>,
	pub class: Option<Uuid>,
	pub school_id: Option<Uuid>,
}