use serde::Deserialize;
use uuid::Uuid;

use crate::common::types::Room;

pub type GetRoomResponse = Room;

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub description: String,
    pub building: String,
	pub school_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct PatchRoomRequest {
	pub name: Option<String>,
    pub description: Option<String>,
    pub building: Option<String>,
	pub school_id: Option<Uuid>,
}