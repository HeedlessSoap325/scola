use axum::{Json, extract::{Path, State}, http::StatusCode};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{common::{admin_auth::{is_admin, resolve_school}, error::{AppError, db_error}, ownership::verify_ownership, sql::{create_resource, delete_resource}, state::AppState, types::{GenericResponse, PersonRole, ResourceResponse, Semester}}, routes::{auth::guards::AuthUser, semester::models::{CreateSemesterRequest, GetSemesterResponse, PatchSemesterRequest}}};

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

pub async fn add_semester(
	State(state): State<AppState>,
	user: AuthUser,
	Json(body): Json<CreateSemesterRequest>,
) -> Result<ResourceResponse, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	let semester: Semester = Semester { 
		id: Uuid::new_v4(), 
		school_id: school_id, 
		name: body.name, 
		start_date: body.start_date, 
		end_date: body.end_date,
	};
	create_resource::<Semester>(&state.pool, semester.clone()).await?;
	
	Ok(ResourceResponse(StatusCode::CREATED, semester.id))
}

pub async fn edit_semester(
	State(state): State<AppState>,
	user: AuthUser,
	Path(semester_id): Path<Uuid>,
	Json(body): Json<PatchSemesterRequest>,
) -> Result<GenericResponse, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Semester>(&state.pool, semester_id, school_id).await?;
	}

	sqlx::query(
		r#"
			UPDATE semester
			SET
				name = COALESCE($1, name),
				start_date = COALESCE($2, start_date),
				end_date = COALESCE($3, end_date)
			WHERE id = $4
			RETURNING *
		"#
	)
	.bind(body.name)
	.bind(body.start_date)
	.bind(body.end_date)
	.bind(semester_id)
	.fetch_optional(&state.pool)
	.await
	.map_err(db_error)?
	.ok_or(AppError(StatusCode::NOT_FOUND, "Semester not found"))?;

	Ok(GenericResponse(StatusCode::OK, "Semester updated"))
}

pub async fn delete_semester(
	State(state): State<AppState>,
	user: AuthUser,
	Path(semester_id): Path<Uuid>,
) -> Result<GenericResponse, AppError>
{
	is_admin(&user)?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Semester>(&state.pool, semester_id, user.school_id).await?;
	}

	delete_resource::<Semester>(&state.pool, semester_id).await?;

	Ok(GenericResponse(StatusCode::OK, "Semester deleted"))
}