use axum::{Json, extract::State};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{common::{admin_auth::resolve_school, error::{AppError, db_error}, state::AppState, types::{PersonRole, ResourceResponse, Semester}}, routes::{auth::guards::AuthUser, semester::models::{CreateSemesterRequest, GetSemesterResponse}}};

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