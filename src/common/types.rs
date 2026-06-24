use serde::Serialize;
use sqlx::types::{BigDecimal, Uuid, chrono::{DateTime, NaiveDate, NaiveTime, Utc}};

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct School {
    pub id: Uuid,
    pub name: String,
    pub abbreviation: String,
    pub address: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Room {
    pub id: Uuid,
    pub school_id: Uuid,
    pub name: String,
    pub description: String,
    pub building: String,
}

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, PartialEq)]
#[sqlx(type_name = "person_role", rename_all = "snake_case")]
pub enum PersonRole {
    Student,
    Teacher,
    LocalAdmin,
    Admin,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Person {
    pub id: Uuid,
    pub school_id: Uuid,
    pub email: String,
    pub login_name: String,
    pub first_name: String,
    pub last_name: String,
    pub picture: Option<String>,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub role: PersonRole,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Teacher {
    pub id: Uuid,
    pub address: String,
    pub abbreviation: String,
    pub phone: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Class {
    pub id: Uuid,
    pub school_id: Uuid,
    pub teacher_id: Uuid,
    pub name: String,
    pub abbreviation: String,
    pub description: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Student {
    pub id: Uuid,
    pub class_id: Uuid,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Grade {
    pub id: Uuid,
    pub student_id: Uuid,
    pub value: BigDecimal,
    pub weight: BigDecimal,
    pub date: NaiveTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Course {
    pub id: Uuid,
    pub teacher_id: Uuid,
    pub name: String,
    pub abbreviation: String,
    pub description: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Semester {
    pub id: Uuid,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct ClassToCourse {
    pub class_id: Uuid,
    pub course_id: Uuid,
    pub semester_id: Uuid,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Lesson {
    pub id: Uuid,
    pub course_id: Uuid,
    pub room_id: Uuid,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub day_of_week: i16,
    pub title: String,
}

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, PartialEq)]
#[sqlx(type_name = "absence_status", rename_all = "snake_case")]
pub enum AbsenceStatus {
    Pending,
    Rejected,
    Approved,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Absence {
    pub id: Uuid,
    pub student_id: Uuid,
    pub lesson_id: Uuid,
    pub reason: String,
    pub status: AbsenceStatus,
    pub from_timestamp: DateTime<Utc>,
    pub to_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, PartialEq)]
#[sqlx(type_name = "lesson_status", rename_all = "snake_case")]
pub enum LessonStatus {
    Canceled,
    Moved,
    RoomChange,
    Reservation,
    ModifiedByBlock,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LessonOverride {
    pub id: Uuid,
    pub lesson_id: Uuid,
    pub status: LessonStatus,
    pub course_id: Uuid,
    pub room_id: Uuid,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub title: String,
    pub date: NaiveDate,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub message: String,
}