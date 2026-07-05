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

pub async fn delete_sessions_with_value(
    redis: deadpool_redis::Pool,
    target_value: &str,
) -> Result<u64, AppError> {
    let mut conn: deadpool_redis::Connection = redis.get()
		.await
		.map_err(redis_pool_error)?;

    let mut deleted = 0u64;
    let mut cursor: u64 = 0;

    loop {
        let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg("session:*")
            .arg("COUNT")
            .arg(100)
            .query_async(&mut conn)
            .await
			.map_err(redis_error)?;

        for key in keys {
            let value: Option<String> = conn.get(&key)
				.await
				.map_err(redis_error)?;
            if value.as_deref() == Some(target_value) {
                let del: u64 = conn.del(&key)
					.await
					.map_err(redis_error)?;
                deleted += del;
            }
        }

        cursor = next_cursor;
        if cursor == 0 {
            break;
        }
    }

    Ok(deleted)
}