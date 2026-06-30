use axum::{Json, extract::{Path, State}, http::StatusCode};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{common::{admin_auth::{is_admin, resolve_school}, error::{AppError, db_error}, ownership::verify_ownership, state::AppState, types::{Class, Course, GenericResponse, PersonRole, ResourceResponse, Semester, Teacher}}, routes::{auth::guards::AuthUser, course::models::{CreateCourseRequest, GetCourseRequest, GetCourseResponse, PatchCourseRequest}}, verify_ownerships};

pub async fn get_courses(
	State(state): State<AppState>,
	user: AuthUser,
	Json(body): Json<GetCourseRequest>,
) -> Result<Json<Vec<GetCourseResponse>>, AppError>
{
	let mut builder = QueryBuilder::new(
        r#"
            SELECT
				co.id AS course_id,
                co.name AS course_name,
                co.abbreviation AS course_abbreviation,
				co.description AS course_description,
                tp.first_name AS teacher_first_name,
                tp.last_name AS teacher_last_name,
                tp.email AS teacher_email,
				t.phone AS teacher_phone,
				t.address AS teacher_address,
				cl.name AS class_name
            FROM course co
            JOIN person tp ON tp.id = co.teacher_id
            JOIN teacher t ON t.id = tp.id
			JOIN "classToCourse" ctc ON ctc.course_id = co.id AND ctc.semester_id = 
        "#
    );

	builder.push_bind(body.semester_id);
	builder.push(" JOIN class cl ON cl.id = ctc.class_id ");

	match user.role {
        PersonRole::Student => {
			builder.push(" WHERE co.id IN (SELECT course_id FROM \"classToCourse\" WHERE semester_id = ");
			builder.push_bind(body.semester_id);
			builder.push(" AND class_id = (SELECT class_id FROM student WHERE id = ");
			builder.push_bind(user.id);
			builder.push("))");
        }
        PersonRole::Teacher => {
            builder.push(" WHERE tp.email = ");
            builder.push_bind(user.email);
        }
        PersonRole::LocalAdmin => {
			builder.push(" WHERE co.school_id = ");
            builder.push_bind(user.school_id);
        }
		PersonRole::Admin => {} // no filter
    };

	let courses = builder
		.build_query_as::<GetCourseResponse>()
		.fetch_all(&state.pool)
		.await
		.map_err(db_error)?;
	
	Ok(Json(courses))
}

pub async fn add_course(
	State(state): State<AppState>,
	user: AuthUser,
	Json(body): Json<CreateCourseRequest>,
) -> Result<ResourceResponse, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownerships!(
			&state.pool, school_id,
			Teacher => body.teacher,
			Class => body.class,
			Semester => body.semester,
		);
	}

	let course: Course = sqlx::query_as::<_, Course>(
		r#"
			INSERT INTO course
			(id, teacher_id, school_id, name, abbreviation, description)
			VALUES
			($1, $2, $3, $4, $5, $6)
			RETURNING *
		"#
	)
	.bind(Uuid::new_v4())
	.bind(body.teacher)
	.bind(school_id)
	.bind(body.name)
	.bind(body.abbreviation)
	.bind(body.description)
	.fetch_one(&state.pool)
	.await
	.map_err(db_error)?;

	sqlx::query(
		r#"
			INSERT INTO "classToCourse"
			(class_id, course_id, semester_id)
			VALUES
			($1, $2, $3)
		"#
	)
	.bind(body.class)
	.bind(course.id)
	.bind(body.semester)
	.execute(&state.pool)
	.await
	.map_err(db_error)?;

	Ok(ResourceResponse(StatusCode::CREATED, course.id))
}

pub async fn delete_course(
	State(state): State<AppState>,
	user: AuthUser,
	Path(course_id): Path<Uuid>,
) -> Result<GenericResponse, AppError>
{
	is_admin(&user)?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Course>(&state.pool, course_id, user.school_id).await?;
	}

	sqlx::query(
		r#"
			DELETE FROM course c
			WHERE c.id = $1
			RETURNING *
		"#
	)
	.bind(course_id)
	.fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::NOT_FOUND, "Course not found"))?;

	Ok(GenericResponse(StatusCode::OK, "Course deleted"))
}

pub async fn edit_course(
	State(state): State<AppState>,
	user: AuthUser,
	Path(course_id): Path<Uuid>,
	Json(body): Json<PatchCourseRequest>
) -> Result<GenericResponse, AppError>
{
	let school_id = resolve_school(&user, body.school_id, &state.pool).await?;

    if user.role == PersonRole::LocalAdmin {
        verify_ownerships!(
            &state.pool, school_id,
            Course => course_id,
        );

		// if teacher is being changed, verify the new teacher too
        if let Some(teacher_id) = body.teacher {
            verify_ownership::<Teacher>(&state.pool, teacher_id, school_id).await?;
        }

		// if class is being changed, verify the new class too
		if let Some(class_id) = body.class {
            verify_ownership::<Class>(&state.pool, class_id, school_id).await?;
        }
    }

	sqlx::query(
        r#"
            UPDATE course
            SET
                name         = COALESCE($1, name),
                abbreviation = COALESCE($2, abbreviation),
                description  = COALESCE($3, description),
                teacher_id   = COALESCE($4, teacher_id)
            WHERE id = $5
			RETURNING *
        "#,
    )
    .bind(body.name)
    .bind(body.abbreviation)
    .bind(body.description)
    .bind(body.teacher)
    .bind(course_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::NOT_FOUND, "Course not found"))?;

	sqlx::query(
        r#"
            UPDATE "classToCourse"
            SET
                class_id = COALESCE($1, class_id)
            WHERE course_id = $2
			RETURNING *
        "#,
    )
    .bind(body.class)
    .bind(course_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::NOT_FOUND, "ClassToCourse entry not found"))?;

	Ok(GenericResponse(StatusCode::OK, "Course updated"))
}