use axum::{Json, extract::{Path, State}, http::StatusCode};
use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;
use sqlx::{QueryBuilder, encode::IsNull::No};
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

pub async fn add_teacher(
	State(state): State<AppState>,
	user: AuthUser,
	Json(body): Json<CreateTeacherRequest>,
) -> Result<ResourceResponse, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	let existing = sqlx::query(
		"SELECT * FROM person WHERE login_name = $1 AND school_id = $2"
	)
	.bind(body.login_name.clone())
	.bind(school_id)
	.fetch_optional(&state.pool)
	.await
	.map_err(db_error)?;

	if existing.is_some() {
		return Err(AppError(StatusCode::BAD_REQUEST, "User with same login name already exists in the school"));
	}

	let password = hash(body.password, DEFAULT_COST)
		.map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "Hashing of password failed"))?;
	let person: Person = Person { 
		id: Uuid::new_v4(), 
		school_id: school_id,
		email: body.email,
		login_name: body.login_name,
		first_name: body.first_name,
		last_name: body.last_name,
		picture: body.picture,
		password: password,
		created_at: Utc::now(),
		role: PersonRole::Teacher,
	};
	create_resource::<Person>(&state.pool, person.clone()).await?;

	let teacher: Teacher = Teacher { 
		id: person.id,
		address: body.address,
		abbreviation: body.abbreviation,
		phone: body.phone,
	};
	create_resource::<Teacher>(&state.pool, teacher.clone()).await?;
	
	Ok(ResourceResponse(StatusCode::CREATED, person.id))
}

pub async fn edit_teacher(
	State(state): State<AppState>,
	user: AuthUser,
	Path(teacher_id): Path<Uuid>,
	Json(body): Json<PatchTeacherRequest>,
) -> Result<GenericResponse, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Teacher>(&state.pool, teacher_id, school_id).await?;
	}

	sqlx::query(
		r#"
			UPDATE teacher
			SET
				address = COALESCE($1, address),
				abbreviation = COALESCE($2, abbreviation),
				phone = COALESCE($3, phone)
			WHERE id = $4
			RETURNING *
		"#
	)
	.bind(body.address)
	.bind(body.abbreviation)
	.bind(body.phone)
	.bind(teacher_id)
	.fetch_optional(&state.pool)
	.await
	.map_err(db_error)?
	.ok_or(AppError(StatusCode::NOT_FOUND, "Teacher not found"))?;

	let password: Option<String> = match body.password {
		Some(p) => {
			let hash = hash(p, DEFAULT_COST)
				.map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "Hashing of password failed"))?;
			Some(hash)
		},
		None => None,
	};

	sqlx::query(
		r#"
			UPDATE person
			SET
				email = COALESCE($1, email),
				login_name = COALESCE($2, login_name),
				first_name = COALESCE($3, first_name),
				last_name = COALESCE($4, last_name),
				picture = COALESCE($5, picture),
				password = COALESCE($6, password)
			WHERE id = $7
			RETURNING *
		"#
	)
	.bind(body.email)
	.bind(body.login_name)
	.bind(body.first_name)
	.bind(body.last_name)
	.bind(body.picture)
	.bind(password)
	.bind(teacher_id)
	.fetch_optional(&state.pool)
	.await
	.map_err(db_error)?
	.ok_or(AppError(StatusCode::NOT_FOUND, "Person not found"))?;

	Ok(GenericResponse(StatusCode::OK, "Teacher updated"))
}

pub async fn delete_teacher(
	State(state): State<AppState>,
	user: AuthUser,
	Path(teacher_id): Path<Uuid>,
) -> Result<GenericResponse, AppError>
{
	is_admin(&user)?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Teacher>(&state.pool, teacher_id, user.school_id).await?;
	}

	delete_resource::<Teacher>(&state.pool, teacher_id).await?;

	Ok(GenericResponse(StatusCode::OK, "Teacher deleted"))
}