use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{error::{AppError, db_error}, types::{Class, Course, Room, Semester, Student, Teacher}};

/// Implement this for any resource type to verify
pub trait SchoolOwned {
    /// The SQL query to retrieve school_id for a given id.
    /// Must have exactly one parameter ($1 = the resource id).
    fn school_id_query() -> &'static str;
}

impl SchoolOwned for Teacher {
    fn school_id_query() -> &'static str {
        "SELECT p.school_id FROM teacher t JOIN person p ON p.id = t.id WHERE t.id = $1"
    }
}
impl SchoolOwned for Class {
    fn school_id_query() -> &'static str {
        "SELECT school_id FROM class WHERE id = $1"
    }
}
impl SchoolOwned for Semester {
    fn school_id_query() -> &'static str {
        "SELECT school_id FROM semester WHERE id = $1"
    }
}
impl SchoolOwned for Room {
    fn school_id_query() -> &'static str {
        "SELECT school_id FROM room WHERE id = $1"
    }
}
impl SchoolOwned for Course {
    fn school_id_query() -> &'static str {
        "SELECT school_id FROM course WHERE id = $1"
    }
}

impl SchoolOwned for Student {
    fn school_id_query() -> &'static str {
        "SELECT school_id FROM person WHERE id = $1"
    }
}

/// Verify that a single resource belongs to the expected school.
/// Returns NOT_FOUND if the id doesn't exist, FORBIDDEN if school doesn't match.
pub async fn verify_ownership<T: SchoolOwned>(
    pool: &PgPool,
    resource_id: Uuid,
    expected_school_id: Uuid,
) -> Result<(), AppError> {
    let school_id: Uuid = sqlx::query_scalar(T::school_id_query())
        .bind(resource_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?
        .ok_or(AppError(StatusCode::NOT_FOUND, "Resource not found"))?;

    if school_id != expected_school_id {
        return Err(AppError(StatusCode::FORBIDDEN, "Resource does not belong to this school"));
    }

    Ok(())
}

/// Verify multiple resources at once. Fails fast on the first mismatch.
/// Pass a slice of (resource_id, label) pairs for each type.
/// 
/// Usage:
///   verify_ownerships!(pool, school_id,
///       Teacher => body.teacher_id,
///       Class   => body.class_id,
///       Semester => body.semester_id,
///   )
#[macro_export]
macro_rules! verify_ownerships {
    ($pool:expr, $school_id:expr, $( $T:ty => $id:expr ),+ $(,)?) => {{
        $(
            $crate::common::ownership::verify_ownership::<$T>($pool, $id, $school_id).await?;
        )+
    }};
}