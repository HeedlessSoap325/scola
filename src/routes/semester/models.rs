use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::common::types::Semester;

pub type GetSemesterResponse = Semester;

#[derive(Deserialize)]
pub struct CreateSemesterRequest {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
	pub school_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct PatchSemesterRequest {
    pub name: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
	pub school_id: Option<Uuid>,
}