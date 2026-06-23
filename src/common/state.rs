use std::{collections::HashMap, sync::Arc};

use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::sync::RwLock;
use tower_cookies::Key;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cookie_key: Arc<Key>,
	pub sessions: Arc<RwLock<HashMap<String, Uuid>>>
}

impl AppState {
	pub async fn new() -> Self {
		let secret = std::env::var("COOKIE_SECRET")
            .expect("COOKIE_SECRET must be set (min 64 bytes)");

		let db_url = std::env::var("DATABASE_URL")
			.expect("DATABASE_URL must be set");
		
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


		Self {
			pool: pool.clone(),
			cookie_key: Arc::new(Key::from(secret.as_bytes())),
			sessions: Arc::new(RwLock::new(HashMap::new())),
		}
	}
}