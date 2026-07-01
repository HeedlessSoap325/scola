use axum::{Json, extract::{Path, State}, http::StatusCode};
use sqlx::{QueryBuilder};
use uuid::Uuid;

use crate::{common::{admin_auth::{is_admin, resolve_school}, error::{AppError, db_error}, ownership::verify_ownership, sql::{create_resource, delete_resource}, state::AppState, types::{Class, GenericResponse, PersonRole, ResourceResponse, Teacher}}, routes::{auth::guards::AuthUser, class::models::{CreateClassRequest, GetClassResponse, PatchClassRequest}}};

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
) -> Result<ResourceResponse, AppError> {
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	let class = Class {
		id: Uuid::new_v4(),
		school_id: school_id,
		teacher_id: body.teacher,
		name: body.name,
		abbreviation: body.abbreviation,
		description: body.description,
	};
	create_resource::<Class>(&state.pool, class.clone()).await?;

	Ok(ResourceResponse(StatusCode::CREATED, class.id))
}

pub async fn delete_class(
	State(state): State<AppState>,
	user: AuthUser,
	Path(class_id): Path<Uuid>,
) -> Result<GenericResponse, AppError> {
	is_admin(&user)?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Class>(&state.pool, class_id, user.school_id).await?
	}

	delete_resource::<Class>(&state.pool, class_id).await?;

	Ok(GenericResponse(StatusCode::OK, "Class deleted"))
}

pub async fn edit_class(
	State(state): State<AppState>,
	user: AuthUser,
	Path(class_id): Path<Uuid>,
	Json(body): Json<PatchClassRequest>
) -> Result<GenericResponse, AppError> {
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Class>(&state.pool, class_id, school_id).await?;

		if let Some(teacher_id) = body.teacher {
            verify_ownership::<Teacher>(&state.pool, teacher_id, school_id).await?;
        }
	}

	sqlx::query(
		r#"
			UPDATE class c
			SET
				teacher_id   = COALESCE($1, teacher_id),
				name         = COALESCE($2, name),
				abbreviation = COALESCE($3, abbreviation),
				description  = COALESCE($4, description)	
			WHERE c.id = $5
		"#
	)
	.bind(body.teacher)
	.bind(body.name)
	.bind(body.abbreviation)
	.bind(body.description)
	.bind(class_id)
	.execute(&state.pool)
	.await
	.map_err(db_error)?;

	Ok(GenericResponse(StatusCode::OK, "Class updated"))
}