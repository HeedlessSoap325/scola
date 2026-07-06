use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{common::{admin_auth::resolve_school, error::{AppError, db_error}, sql::create_resource, state::AppState, types::{Grade, PersonRole, ResourceResponse, Student}}, routes::{auth::guards::AuthUser, grade::models::{CreateGradeRequest, GetGradeResponse}}, verify_ownerships};

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
	// TODO: Actually allow Teachers to create grades...
	// But currently, there is no teacher id for grades, so they cant be associated...
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownerships!(
			&state.pool, school_id,
			Student => body.student_id,
		);
	}

	let grade: Grade = Grade { 
		id: Uuid::new_v4(), 
		student_id: body.student_id, 
		value: body.value,
		weight: body.weight, 
		date: body.date,
		// TODO: add description & teacher_id 
	};
	create_resource::<Grade>(&state.pool, grade.clone()).await?;

	Ok(ResourceResponse(StatusCode::CREATED, grade.id))
}