use axum::{Json, extract::{Path, State}, http::StatusCode};
use uuid::Uuid;

use crate::{common::{error::{AppError, db_error}, ownership::verify_ownership, sql::{create_resource, delete_resource, get_resource}, state::AppState, types::{Course, GenericResponse, Grade, PersonRole, ResourceResponse, Student}}, routes::{auth::guards::AuthUser, grade::models::{CreateGradeRequest, GetGradeResponse, PatchGradeRequest}}, verify_ownerships};

pub async fn get_grades(
	State(state): State<AppState>,
	user: AuthUser
) -> Result<Json<Vec<GetGradeResponse>>, AppError>
{
	if user.role != PersonRole::Student {
		return Err(AppError(StatusCode::BAD_REQUEST, "Only students can get grades"));
	}

	let grades: Vec<GetGradeResponse> = sqlx::query_as::<_, GetGradeResponse>(
		r#"
			SELECT * FROM grade
			WHERE student_id = $1
		"#
	)
	.bind(user.id)
	.fetch_all(&state.pool)
	.await
	.map_err(db_error)?;

	Ok(Json(grades))
}

pub async fn add_grade(
	State(state): State<AppState>,
	user: AuthUser,
	Json(body): Json<CreateGradeRequest>,
) -> Result<ResourceResponse, AppError>
{
	match user.role {
		PersonRole::Student => {
			return Err(AppError(StatusCode::UNAUTHORIZED, "Insufficient Privileges"))
		},
		PersonRole::Teacher | PersonRole::LocalAdmin => {
			verify_ownerships!(
				&state.pool, user.school_id,
				Student => body.student_id,
				Course => body.course_id,
			);

			if user.role == PersonRole::Teacher {
				let course: Course = get_resource::<Course>(&state.pool, body.course_id).await?;
				if course.teacher_id != user.id {
					return Err(AppError(StatusCode::BAD_REQUEST, "You are not a teacher of that course"))
				}
			}
		},
		PersonRole::Admin => {}
	}

	let grade: Grade = Grade { 
		id: Uuid::new_v4(), 
		student_id: body.student_id, 
		value: body.value,
		weight: body.weight, 
		date: body.date,
		description: body.description,
		course_id: body.course_id,
	};
	create_resource::<Grade>(&state.pool, grade.clone()).await?;

	Ok(ResourceResponse(StatusCode::CREATED, grade.id))
}

pub async fn edit_grade(
	State(state): State<AppState>,
	user: AuthUser,
	Path(grade_id): Path<Uuid>,
	Json(body): Json<PatchGradeRequest>,
) -> Result<GenericResponse, AppError>
{
	match user.role {
		PersonRole::Student => {
			return Err(AppError(StatusCode::UNAUTHORIZED, "Insufficient Privileges"))
		},
		PersonRole::Teacher | PersonRole::LocalAdmin => {
			verify_ownership::<Grade>(&state.pool, grade_id, user.school_id).await?;

			if user.role == PersonRole::Teacher {
				let grade: Grade = get_resource::<Grade>(&state.pool, grade_id).await?;
				let course: Course = get_resource::<Course>(&state.pool, grade.course_id).await?;

				if course.teacher_id != user.id {
					return Err(AppError(StatusCode::BAD_REQUEST, "You are not a teacher of that course"))
				}
			}
		},
		PersonRole::Admin => {}
	}

	sqlx::query(
        r#"
            UPDATE grade
            SET
				student_id = COALESCE($1, student_id),
				course_id = COALESCE($2, course_id),
				value = COALESCE($3, value),
				weight = COALESCE($4, weight),
				date = COALESCE($5, date),
				description = COALESCE($6, description)
            WHERE id = $7
			RETURNING *
        "#,
    )
    .bind(body.student_id)
    .bind(body.course_id)
	.bind(body.value)
	.bind(body.weight)
	.bind(body.date)
	.bind(body.description)
	.bind(grade_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::NOT_FOUND, "Grade not found"))?;

	Ok(GenericResponse(StatusCode::OK, "Grade updated"))
}

pub async fn delete_grade(
	State(state): State<AppState>,
	user: AuthUser,
	Path(grade_id): Path<Uuid>,
) -> Result<GenericResponse, AppError>
{
	match user.role {
		PersonRole::Student => {
			return Err(AppError(StatusCode::UNAUTHORIZED, "Insufficient Privileges"))
		},
		PersonRole::Teacher | PersonRole::LocalAdmin => {
			verify_ownership::<Grade>(&state.pool, grade_id, user.school_id).await?;

			if user.role == PersonRole::Teacher {
				let grade: Grade = get_resource::<Grade>(&state.pool, grade_id).await?;
				let course: Course = get_resource::<Course>(&state.pool, grade.course_id).await?;

				if course.teacher_id != user.id {
					return Err(AppError(StatusCode::BAD_REQUEST, "You are not a teacher of that course"))
				}
			}
		},
		PersonRole::Admin => {}
	};

	delete_resource::<Grade>(&state.pool, grade_id).await?;

	Ok(GenericResponse(StatusCode::OK, "Grade deleted"))
}