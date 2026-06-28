use axum::{Json, extract::{Path, State}, http::StatusCode};
use uuid::Uuid;

use crate::{common::{error::{AppError, db_error}, state::{self, AppState}, types::{GenericResponse, PersonRole, ResourceResponse, School}}, routes::{auth::guards::AuthUser, class::models::PatchClassRequest, school::{self, models::{CreateSchoolRequest, GetSchoolResponse, PatchSchoolRequest}}}};

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
) -> Result<Json<ResourceResponse>, AppError>
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

	Ok(Json(ResourceResponse { 
		resource_id: school.id 
	}))
}

pub async fn edit_school(
	State(state): State<AppState>,
	user: AuthUser,
	Path(school_id): Path<Uuid>,
	Json(body): Json<PatchSchoolRequest>,
) -> Result<Json<GenericResponse>, AppError>
{
	if user.role != PersonRole::Admin {
		return Err(AppError( StatusCode::UNAUTHORIZED, "Insufficient privileges" ));
	}
	
	sqlx::query(
        r#"
            UPDATE school
            SET
                name = COALESCE($1, name),
                abbreviation = COALESCE($2, abbreviation),
                address = COALESCE($3, address)
            WHERE id = $4
			RETURNING *
        "#,
    )
    .bind(body.name)
    .bind(body.abbreviation)
	.bind(body.address)
	.bind(school_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::NOT_FOUND, "School not found"))?;

	Ok(Json(GenericResponse { 
		message: "School updated".to_string(),
	}))
}

pub async fn delete_school(
	State(state): State<AppState>,
	user: AuthUser,
	Path(school_id): Path<Uuid>,
) -> Result<Json<GenericResponse>, AppError>
{
	if user.role != PersonRole::Admin {
		return Err(AppError( StatusCode::UNAUTHORIZED, "Insufficient privileges" ));
	}

	sqlx::query(
		r#"
			DELETE FROM school s
			WHERE s.id = $1
			RETURNING *
		"#
	)
	.bind(school_id)
	.fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::NOT_FOUND, "School not found"))?;

	Ok(Json(GenericResponse { 
		message: "School deleted".to_string(),
	}))
}