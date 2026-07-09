use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::common::types::Grade;

pub type GetGradeResponse = Grade;

#[derive(Deserialize)]
pub struct CreateGradeRequest {
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub value: BigDecimal,
    pub weight: BigDecimal,
    pub date: NaiveDate,
    pub description: String,
}

#[derive(Deserialize)]
pub struct PatchGradeRequest {
    pub student_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub value: Option<BigDecimal>,
    pub weight: Option<BigDecimal>,
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
}