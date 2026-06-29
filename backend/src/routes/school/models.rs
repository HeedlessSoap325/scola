use serde::Deserialize;

use crate::common::types::School;

pub type GetSchoolResponse = School;

#[derive(Deserialize)]
pub struct CreateSchoolRequest {
    pub name: String,
    pub abbreviation: String,
    pub address: String,
}

#[derive(Deserialize)]
pub struct PatchSchoolRequest {
	pub name: Option<String>,
    pub abbreviation: Option<String>,
    pub address: Option<String>,
}