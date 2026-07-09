use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{common::{error::{AppError, db_error}, sql::{create_resource, get_resource}, state::AppState, types::{Course, Grade, PersonRole, ResourceResponse, Student}}, routes::{auth::guards::AuthUser, grade::models::{CreateGradeRequest, GetGradeResponse}}, verify_ownerships};

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