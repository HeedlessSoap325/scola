use axum::{Json, extract::State};
use sqlx::QueryBuilder;

use crate::{common::{error::{AppError, db_error}, state::AppState, types::PersonRole}, routes::{auth::guards::AuthUser, room::models::GetRoomResponse}};

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