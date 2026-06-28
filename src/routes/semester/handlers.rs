use axum::{Json, extract::State};
use sqlx::QueryBuilder;

use crate::{common::{error::{AppError, db_error}, state::AppState, types::PersonRole}, routes::{auth::guards::AuthUser, semester::models::GetSemesterResponse}};

pub async fn get_semesters(
	State(state): State<AppState>,
	user: AuthUser,
) -> Result<Json<Vec<GetSemesterResponse>>, AppError>
{
	let mut builder = QueryBuilder::new(
        r#"
            SELECT *
            FROM semester s
        "#
    );

	match user.role {
        PersonRole::Student | PersonRole::Teacher | PersonRole::LocalAdmin => {
			builder.push(" WHERE s.school_id = ");
			builder.push_bind(user.school_id);
        }
		PersonRole::Admin => {} // no filter
    };

	let semesters: Vec<GetSemesterResponse> = builder
		.build_query_as::<GetSemesterResponse>()
		.fetch_all(&state.pool)
		.await
		.map_err(db_error)?;

	Ok(Json(semesters))
}