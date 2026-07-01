use axum::{Json, extract::{Path, State}, http::StatusCode};
use uuid::Uuid;

use crate::{common::{error::{AppError, db_error}, sql::{create_resource, delete_resource}, state::AppState, types::{GenericResponse, PersonRole, ResourceResponse, School}}, routes::{auth::guards::AuthUser, school::models::{CreateSchoolRequest, GetSchoolResponse, PatchSchoolRequest}}};

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
) -> Result<ResourceResponse, AppError>
{
	if user.role != PersonRole::Admin {
		return Err(AppError( StatusCode::UNAUTHORIZED, "Insufficient privileges" ));
	}

	let school: School = School { 
		id: Uuid::new_v4(), 
		name: body.name, 
		abbreviation: body.abbreviation, 
		address: body.address,
	};
	create_resource::<School>(&state.pool, school.clone()).await?;

	Ok(ResourceResponse(StatusCode::CREATED, school.id))
}

pub async fn edit_school(
	State(state): State<AppState>,
	user: AuthUser,
	Path(school_id): Path<Uuid>,
	Json(body): Json<PatchSchoolRequest>,
) -> Result<GenericResponse, AppError>
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

	Ok(GenericResponse(StatusCode::OK, "School updated"))
}

pub async fn delete_school(
	State(state): State<AppState>,
	user: AuthUser,
	Path(school_id): Path<Uuid>,
) -> Result<GenericResponse, AppError>
{
	if user.role != PersonRole::Admin {
		return Err(AppError( StatusCode::UNAUTHORIZED, "Insufficient privileges" ));
	}

	delete_resource::<School>(&state.pool, school_id).await?;

	Ok(GenericResponse(StatusCode::OK, "School deleted"))
}