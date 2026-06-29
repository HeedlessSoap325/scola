use axum::{Json, extract::State};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{common::{admin_auth::resolve_school, error::{AppError, db_error}, state::AppState, types::{PersonRole, ResourceResponse, Room}}, routes::{auth::guards::AuthUser, room::models::{CreateRoomRequest, GetRoomResponse}}};

pub async fn get_rooms(
	State(state): State<AppState>,
	user: AuthUser
) -> Result<Json<Vec<GetRoomResponse>>, AppError>
{
	let mut builder = QueryBuilder::new(
        r#"
			SELECT * FROM room
        "#
    );

	match user.role {
		PersonRole::Student | PersonRole::Teacher | PersonRole::LocalAdmin => {
			builder.push(" WHERE school_id = ");
			builder.push_bind(user.school_id);
		}
		PersonRole::Admin => {} // no filter
	};

	let rooms: Vec<GetRoomResponse> = builder
		.build_query_as::<GetRoomResponse>()
		.fetch_all(&state.pool)
		.await
		.map_err(db_error)?;

	Ok(Json(rooms))
}

pub async fn add_room(
	State(state): State<AppState>,
	user: AuthUser,
	Json(body): Json<CreateRoomRequest>,
) -> Result<Json<ResourceResponse>, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	let room = sqlx::query_as::<_, Room>(
		r#"
			INSERT INTO room
			(id, school_id, name, description, building)
			VALUES
			($1, $2, $3, $4, $5)
			RETURNING *
		"#
	)
	.bind(Uuid::new_v4())
	.bind(school_id)
	.bind(body.name)
	.bind(body.description)
	.bind(body.building)
	.fetch_one(&state.pool)
	.await
	.map_err(db_error)?;


	Ok(Json(ResourceResponse { 
		resource_id: room.id 
	}))
}