use axum::{Json, extract::{Path, State}, http::StatusCode};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{common::{admin_auth::{is_admin, resolve_school}, error::{AppError, db_error}, ownership::verify_ownership, sql::{create_resource, delete_resource}, state::AppState, types::{GenericResponse, PersonRole, ResourceResponse, Room}}, routes::{auth::guards::AuthUser, room::models::{CreateRoomRequest, GetRoomResponse, PatchRoomRequest}}};

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
) -> Result<ResourceResponse, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;

	let room: Room = Room {
		id: Uuid::new_v4(),
		school_id: school_id,
		name: body.name,
		description: body.description,
		building: body.building,
	};
	create_resource::<Room>(&state.pool, room.clone()).await?;

	Ok(ResourceResponse(StatusCode::CREATED, room.id))
}

pub async fn edit_room(
	State(state): State<AppState>,
	user: AuthUser,
	Path(room_id): Path<Uuid>,
	Json(body): Json<PatchRoomRequest>,
) -> Result<GenericResponse, AppError>
{
	let school_id: Uuid = resolve_school(&user, body.school_id, &state.pool).await?;
	
	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Room>(&state.pool, room_id, school_id).await?;
	}

	sqlx::query(
		r#"
			UPDATE room
			SET
				name = COALESCE($1, name),
				description = COALESCE($2, description),
				building = COALESCE($3, building)
			WHERE id = $4
			RETURNING *
		"#
	)
	.bind(body.name)
	.bind(body.description)
	.bind(body.building)
	.bind(room_id)
	.fetch_optional(&state.pool)
	.await
	.map_err(db_error)?
	.ok_or(AppError(StatusCode::NOT_FOUND, "Room entry not found"))?;

	Ok(GenericResponse(StatusCode::OK, "Room updated"))
}

pub async fn delete_room(
	State(state): State<AppState>,
	user: AuthUser,
	Path(room_id): Path<Uuid>,
) -> Result<GenericResponse, AppError>
{
	is_admin(&user)?;

	if user.role == PersonRole::LocalAdmin {
		verify_ownership::<Room>(&state.pool, room_id, user.school_id).await?;
	}

	delete_resource::<Room>(&state.pool, room_id).await?;

	Ok(GenericResponse(StatusCode::OK, "Room deleted"))
}