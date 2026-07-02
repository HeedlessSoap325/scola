use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::types::LessonStatus;

#[derive(Deserialize)]
pub struct GetTimetableRequest {
	pub semester_id: Uuid,
	pub start: NaiveDate,
	pub end: NaiveDate,
}

#[derive(sqlx::FromRow, Serialize, Default)]
pub struct GetTimetableResponse {
	pub lesson_id: Uuid,
	pub lesson_date: NaiveDate,
	pub lesson_status: Option<LessonStatus>,
	pub lesson_start: NaiveTime,
	pub lesson_end: NaiveTime,

	pub room_id: Uuid,
	pub room_name: String,

	pub course_id: Uuid,
	pub course_name: String,
	pub course_abbreviation: String,

	pub teacher_id: Uuid,
	pub teacher_abbreviation: String,
	pub teacher_first_name: String,
	pub teacher_last_name: String,

	pub class_id: Uuid,
	pub class_abbreviation: String,
	pub class_name: String,
}