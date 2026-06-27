use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{common::{error::{AppError, db_error}, state::AppState, types::{PersonRole, RessourceResponse, School}}, routes::{auth::guards::AuthUser, school::models::{CreateSchoolRequest, GetSchoolResponse}}};

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

pub async fn add_school(
	State(state): State<AppState>,
	user: AuthUser,
	Json(body): Json<CreateSchoolRequest>,
) -> Result<Json<RessourceResponse>, AppError>
{
	if user.role != PersonRole::Admin {
		return Err(AppError( StatusCode::UNAUTHORIZED, "Insufficient privileges" ));
	}

	let school = sqlx::query_as::<_, School>(
		r#"
			INSERT INTO school
			(id, name, abbreviation, address)
			VALUES
			($1, $2, $3, $4)
			RETURNING *
		"#
	)
	.bind(Uuid::new_v4())
	.bind(body.name)
	.bind(body.abbreviation)
	.bind(body.address)
	.fetch_one(&state.pool)
	.await
	.map_err(db_error)?;

	Ok(Json(RessourceResponse { 
		ressource_id: school.id 
	}))
}