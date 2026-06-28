use axum::{Json, extract::{Path, State}, http::StatusCode};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{common::{admin_auth::{is_admin, resolve_school}, error::{AppError, db_error}, ownership::verify_ownership, state::AppState, types::{GenericResponse, PersonRole, ResourceResponse, Semester}}, routes::{auth::guards::AuthUser, semester::models::{CreateSemesterRequest, GetSemesterResponse, PatchSemesterRequest}}};

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
) -> Result<Json<ResourceResponse>, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	let semester: Semester = sqlx::query_as::<_, Semester>(
		r#"
			INSERT INTO semester
			(id, school_id, name, start_date, end_date)
			VALUES
			($1, $2, $3, $4, $5)
			RETURNING *
		"#
	)
	.bind(Uuid::new_v4())
	.bind(school_id)
	.bind(body.name)
	.bind(body.start_date)
	.bind(body.end_date)
	.fetch_one(&state.pool)
	.await
	.map_err(db_error)?;
	
	Ok(Json(ResourceResponse { 
		resource_id: semester.id,
	}))
}

pub async fn edit_semester(
	State(state): State<AppState>,
	user: AuthUser,
	Path(semester_id): Path<Uuid>,
	Json(body): Json<PatchSemesterRequest>,
) -> Result<Json<GenericResponse>, AppError>
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

	Ok(Json(GenericResponse { 
		message: "Semester updated".to_string(),
	}))
}

pub async fn delete_semester(
	State(state): State<AppState>,
	user: AuthUser,
	Path(semester_id): Path<Uuid>,
) -> Result<Json<GenericResponse>, AppError>
{
	is_admin(&user)?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Semester>(&state.pool, semester_id, user.school_id).await?;
	}

	sqlx::query(
		r#"
			DELETE FROM semester s
			WHERE s.id = $1
			RETURNING *
		"#
	)
	.bind(semester_id)
	.fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::NOT_FOUND, "Semester not found"))?;

	Ok(Json(GenericResponse { 
		message: "Semester deleted".to_string(),
	}))
}