use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use deadpool_redis::{Config, Runtime};
use redis::AsyncCommands;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::sync::RwLock;
use tower_cookies::Key;
use uuid::Uuid;

#[derive(Clone)]
pub struct Session {
	pub user_id: Uuid,
	expiry: DateTime<Utc>,
}

impl Session {
	pub fn new(user_id: Uuid) -> Self {
		Self { user_id, expiry: Utc::now() + Duration::days(1) }
	}

	pub fn valid(&self) -> bool {
		Utc::now() < self.expiry
	}
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cookie_key: Arc<Key>,
	pub redis: deadpool_redis::Pool,
	pub sessions: Arc<RwLock<HashMap<String, Session>>>
}

impl AppState {
	pub async fn new() -> Self {
		let secret = std::env::var("COOKIE_SECRET")
            .expect("COOKIE_SECRET must be set (min 64 bytes)");

		let db_url = std::env::var("DATABASE_URL")
			.expect("DATABASE_URL must be set");

		let redis_url = std::env::var("REDIS_URL")
			.expect("REDIS_URL must be set");
		
		let pool = match PgPoolOptions::new()
			.max_connections(10)
			.connect(&db_url)
			.await
		{
			Ok(pool) => {
				println!("Connected to DB successfully");
				pool
			}
			Err(err) => {
				println!("Failed to connect to DB: {}", err);
				std::process::exit(1);
			}
		};

		let redis = match Config::from_url(redis_url).create_pool(Some(Runtime::Tokio1)) {
			Ok(pool) => pool,
			Err(err) => {
				eprintln!("Invalid Redis config: {}", err);
				std::process::exit(1);
			}
		};
		
		// Test connection to redis, because create_pool is lazy
		match redis.get().await {
			Ok(_) => {
				println!("Connected to Redis successfully");
			}
			Err(err) => {
				eprintln!("Failed to connect to Redis: {}", err);
				std::process::exit(1);
			}
		};

		Self {
			pool: pool.clone(),
			cookie_key: Arc::new(Key::from(secret.as_bytes())),
			redis: redis.clone(),
			sessions: Arc::new(RwLock::new(HashMap::new())),
		}
	}
}