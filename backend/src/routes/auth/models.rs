use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::types::PersonRole;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub school: Uuid,
    pub login_name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub login_name: String,
    pub first_name: String,
    pub last_name: String,
    pub picture: Option<String>,
    pub role: PersonRole,
    pub created_at: DateTime<Utc>,
}