use axum::{Json, extract::{Path, State}, http::StatusCode};
use sqlx::{QueryBuilder};
use uuid::Uuid;

use crate::{common::{admin_auth::{is_admin, resolve_school}, error::{AppError, db_error}, ownership::verify_ownership, state::AppState, types::{Class, GenericResponse, PersonRole, ResourceResponse, Teacher}}, routes::{auth::guards::AuthUser, class::models::{CreateClassRequest, GetClassResponse, PatchClassRequest}}};

pub async fn get_classes(
	State(state): State<AppState>, 
	user: AuthUser,
) -> Result<Json<Vec<GetClassResponse>>, AppError> 
{
	let mut builder = QueryBuilder::new(
        r#"
            SELECT
                c.name AS class_name,
                c.abbreviation AS class_abbreviation,
                tp.first_name AS teacher_first_name,
                tp.last_name AS teacher_last_name,
                tp.email AS teacher_email,
                JSON_AGG(JSON_BUILD_OBJECT(
                    'first_name', sp.first_name,
                    'last_name',  sp.last_name,
                    'email',      sp.email
                )) AS persons
            FROM class c
            JOIN person tp  ON tp.id = c.teacher_id
            JOIN student s ON s.class_id = c.id
            JOIN person sp ON sp.id = s.id
        "#
    );

	match user.role {
        PersonRole::Student => {
			builder.push(" WHERE c.id = (SELECT class_id FROM student WHERE id = (SELECT id FROM person WHERE email = ");
			builder.push_bind(user.email.clone());
			builder.push("))");
        }
        PersonRole::Teacher => {
            builder.push(" WHERE tp.email = ");
            builder.push_bind(user.email);
        }
        PersonRole::LocalAdmin => {
			builder.push(" WHERE c.school_id = ");
            builder.push_bind(user.school_id);
        }
		PersonRole::Admin => {} // no filter
    };

    builder.push(
		" GROUP BY c.id, c.name, c.abbreviation, tp.id, tp.first_name, tp.last_name, tp.email"
	);

	let calsses = builder
		.build_query_as::<GetClassResponse>()
		.fetch_all(&state.pool)
		.await
		.map_err(db_error)?;

	Ok(Json(calsses))
}

pub async fn add_class(
	State(state): State<AppState>, 
	user: AuthUser, 
	Json(body): Json<CreateClassRequest>,
) -> Result<Json<ResourceResponse>, AppError> {
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	let class: Class = sqlx::query_as::<_, Class>(
		r#"
			INSERT INTO class 
			(id, school_id, teacher_id, name, abbreviation, description) 
			VALUES
			($1, $2, $3, $4, $5, $6)
			RETURNING *
		"#
	)
	.bind(Uuid::new_v4())
	.bind(school_id)
	.bind(body.teacher)
	.bind(body.name)
	.bind(body.abbreviation)
	.bind(body.description)
	.fetch_one(&state.pool)
	.await
	.map_err(db_error)?;

	Ok(Json(ResourceResponse { 
		resource_id: class.id,
	}))
}

pub async fn delete_class(
	State(state): State<AppState>,
	user: AuthUser,
	Path(class_id): Path<Uuid>,
) -> Result<Json<GenericResponse>, AppError> {
	is_admin(&user)?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Class>(&state.pool, class_id, user.school_id).await?
	}

	sqlx::query(
		r#"
			DELETE FROM class c
			WHERE c.id = $1
		"#
	)
	.bind(class_id).
	execute(&state.pool)
	.await
	.map_err(db_error)?;

	Ok(Json( GenericResponse {
		message: "Class deleted".to_string(),
	}))
}

pub async fn edit_class(
	State(state): State<AppState>,
	user: AuthUser,
	Path(class_id): Path<Uuid>,
	Json(body): Json<PatchClassRequest>
) -> Result<Json<GenericResponse>, AppError> {
	let class = sqlx::query_as::<_, Class>(
        r#"
            SELECT 
				* 
			FROM class c
            WHERE c.id = $1
        "#,
    )
    .bind(class_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?;

	if (user.role != PersonRole::LocalAdmin || class.school_id != user.school_id ) && user.role != PersonRole::Admin {
		return Err( AppError(
			StatusCode::UNAUTHORIZED,
			"Your privileges are not sufficient to perform this operation",
		));
	}

	let mut new_class: Class = class.clone();

	if body.name.is_some() {
		new_class.name = body.name.unwrap();
	}

	if body.abbreviation.is_some() {
		new_class.abbreviation = body.abbreviation.unwrap();
	}

	if body.description.is_some() {
		new_class.description = body.description.unwrap();
	}

	if body.teacher.is_some() {
		new_class.teacher_id = body.teacher.unwrap();
	}

	sqlx::query(
		r#"
			UPDATE class c
			SET
				teacher_id = $1,
				name = $2,
				abbreviation = $3,
				description = $4
			WHERE c.id = $5
		"#
	)
	.bind(new_class.teacher_id)
	.bind(new_class.name)
	.bind(new_class.abbreviation)
	.bind(new_class.description)
	.bind(new_class.id)
	.execute(&state.pool)
	.await
	.map_err(db_error)?;

	Ok(Json( GenericResponse { 
		message: "Class updated".to_string()
	}))
}