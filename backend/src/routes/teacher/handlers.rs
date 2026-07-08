use axum::{Json, extract::{Path, State}, http::StatusCode};
use chrono::Utc;
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{common::{admin_auth::{is_admin, resolve_school}, error::{AppError, db_error}, extractors::Filter, ownership::verify_ownership, sql::{create_resource, delete_resource}, state::AppState, types::{GenericResponse, Person, PersonRole, ResourceResponse, Teacher}}, routes::{auth::guards::AuthUser, teacher::models::{CreateTeacherRequest, GetTeacherResponse, PatchTeacherRequest}}};

pub async fn get_teachers(
	filter: Filter<GetTeacherResponse>,
	State(state): State<AppState>,
	user: AuthUser,
) -> Result<Json<Vec<GetTeacherResponse>>, AppError>
{
	let mut builder = QueryBuilder::new(
        r#"WITH teachers AS (
            SELECT 
				p.id as id,
				p.school_id AS school_id,
				p.email AS email,
				p.first_name AS first_name,
				p.last_name AS last_name,
				p.picture AS picture,
				t.address AS address,
				t.abbreviation AS abbreviation,
				t.phone AS phone
            FROM teacher t
			JOIN person p
				ON p.id = t.id"#
    );

	match user.role {
        PersonRole::Student | PersonRole::Teacher | PersonRole::LocalAdmin => {
			builder.push(" WHERE p.school_id = ");
			builder.push_bind(user.school_id);
        }
		PersonRole::Admin => {} // no filter
    };

	builder.push(") SELECT * FROM teachers ");
	if !filter.is_empty() {
		builder.push(" WHERE ");
		filter.apply(&mut builder);
	}

	let teachers: Vec<GetTeacherResponse> = builder
		.build_query_as::<GetTeacherResponse>()
		.fetch_all(&state.pool)
		.await
		.map_err(db_error)?;

	Ok(Json(teachers))
}