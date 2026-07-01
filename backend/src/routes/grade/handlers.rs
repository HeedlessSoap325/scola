use axum::{Json, extract::State, http::StatusCode};

use crate::{common::{error::{AppError, db_error}, state::AppState, types::PersonRole}, routes::{auth::guards::AuthUser, grade::models::GetGradeResponse}};

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