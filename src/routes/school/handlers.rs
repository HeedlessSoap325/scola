use axum::{Json, extract::State};

use crate::{common::{error::{AppError, db_error}, state::AppState}, routes::school::models::GetSchoolResponse};

pub async fn get_schools(
	State(state): State<AppState>,
) -> Result<Json<Vec<GetSchoolResponse>>, AppError>
{
	let schools: Vec<GetSchoolResponse> = sqlx::query_as::<_, GetSchoolResponse>(
		r#"
			SELECT * FROM school
		"#
	)
	.fetch_all(&state.pool)
	.await
	.map_err(db_error)?;

	Ok(Json(schools))
}