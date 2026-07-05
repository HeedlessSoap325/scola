use std::str::FromStr;

use redis::AsyncCommands;
use uuid::Uuid;

use crate::common::error::{AppError, redis_error, redis_pool_error};

const SESSION_TTL: u64 = 60 * 60 * 24; // 24 Hours

pub async fn write_session(
	redis: deadpool_redis::Pool,
	user_id: Uuid
) -> Result<String, AppError>
{
	let mut conn: deadpool_redis::Connection = redis.get()
		.await
		.map_err(redis_pool_error)?;

	let session_id = Uuid::new_v4().to_string();
	let _: () = conn.set_ex(format!("session:{session_id}"), user_id.to_string(), SESSION_TTL)
		.await
		.map_err(redis_error)?;

	Ok(session_id)
}