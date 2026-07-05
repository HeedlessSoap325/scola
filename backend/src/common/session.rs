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

pub async fn get_session(
	redis: deadpool_redis::Pool,
	session_id: String
) -> Result<Option<Uuid>, AppError>
{
	let mut conn: deadpool_redis::Connection = redis.get()
		.await
		.map_err(redis_pool_error)?;

	let val: Option<String> = conn.get(format!("session:{session_id}"))
		.await
		.map_err(redis_error)?;

	match val {
		Some(user_id) => Ok(Some(Uuid::from_str(user_id.as_str()).unwrap())),
		None => Ok(None),
	}
}

pub async fn delete_session(
	redis: deadpool_redis::Pool,
	session_id: String,
) -> Result<(), AppError>
{
	let mut conn: deadpool_redis::Connection = redis.get()
		.await
		.map_err(redis_pool_error)?;

	let _: u64 = conn.del(format!("session:{session_id}"))
		.await
		.map_err(redis_error)?;

	Ok(())
}