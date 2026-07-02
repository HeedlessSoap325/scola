use axum::{Json, extract::State, http::StatusCode};
use sqlx::QueryBuilder;

use crate::{common::{error::{AppError, db_error}, extractors::Filter, ownership::verify_ownership, state::AppState, types::{PersonRole, Semester}}, routes::{auth::guards::AuthUser, timetable::models::{GetTimetableRequest, GetTimetableResponse}}};

pub async fn get_timetable(
	State(state): State<AppState>,
	filter: Filter<GetTimetableResponse>,
	user: AuthUser,
	Json(body): Json<GetTimetableRequest>,
) -> Result<Json<Vec<GetTimetableResponse>>, AppError>
{
	if user.role == PersonRole::Admin {
		println!("[get_timetable] Admin not supported yet!");
		return Err(AppError(StatusCode::INTERNAL_SERVER_ERROR, "Admin not supported yet"));
	}

	verify_ownership::<Semester>(&state.pool, body.semester_id, user.school_id).await?;
	
	// TODO: verify that start and end are inside the semesters bounds

	let mut builder = QueryBuilder::new(r#"
		WITH dates AS (
			SELECT generate_series( "#
	);		
	builder.push_bind(body.start);

	builder.push("::date, ");
	builder.push_bind(body.end);

	builder.push(r#"
				::date,
				interval '1 day'
			)::date as lesson_date
		),
		lessons AS (
			SELECT 
				l.id AS lesson_id,
				d.lesson_date AS lesson_date,
				lov.status AS lesson_status,
				COALESCE(lov.start_time, l.start_time) AS lesson_start,
				COALESCE(lov.end_time, l.end_time) AS lesson_end,
				r.id AS room_id,
				r.name AS room_name,
				c.id AS course_id,
				c.name AS course_name,
				c.abbreviation AS course_abbreviation,
				t.id AS teacher_id,
				t.abbreviation AS teacher_abbreviation,
				tp.first_name AS teacher_first_name,
				tp.last_name AS teacher_last_name,
				cl.id AS class_id,
				cl.abbreviation AS class_abbreviation,
				cl.name AS class_name
			FROM dates d
			JOIN lesson l
				ON l.day_of_week = EXTRACT(ISODOW FROM d.lesson_date) - 1
			LEFT JOIN "lessonOverride" lov
				ON lov.lesson_id = l.id AND lov.date = d.lesson_date
			JOIN room r
				on r.id = COALESCE(lov.room_id, l.room_id)
			JOIN course c
				ON c.id = COALESCE(lov.course_id, l.course_id)
			JOIN teacher t
				ON t.id = c.teacher_id
			JOIN person tp
				ON tp.id = t.id
			JOIN "classToCourse" ctc
				ON ctc.course_id = c.id AND ctc.semester_id =  "#
	);	
	builder.push_bind(body.semester_id);

	builder.push(r#"
			JOIN class cl
				ON cl.id = ctc.class_id
		)
		SELECT * FROM lessons WHERE "#
	);
	filter.apply(&mut builder);

	builder.push(" ORDER BY lesson_date, lesson_start");

	let lessons: Vec<GetTimetableResponse> = builder
		.build_query_as::<GetTimetableResponse>()
		.fetch_all(&state.pool)
		.await
		.map_err(db_error)?;

	Ok(Json(lessons))
}