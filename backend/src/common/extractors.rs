use axum::{
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;
use sqlx::{Postgres, QueryBuilder};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::common::state::AppState;

pub struct Filter<T: Serialize> {
    conditions: Vec<(String, String)>,
    _marker: PhantomData<T>,
}

impl<T: Serialize> Filter<T> {
    /// Appends `field1::text LIKE $1 AND field2::text LIKE $2 ...`
    /// Does nothing if no query params matched a filterable field.
	/// 
	/// Must be called inside a WHERE clause to work as expected
    pub fn apply<'a>(&'a self, qb: &mut QueryBuilder<Postgres>) {
        if self.conditions.is_empty() {
            return;
        }
        
        let mut sep = qb.separated(" AND ");
        for (field, value) in &self.conditions {
            sep.push(format!("{field}::text LIKE "));
            sep.push_bind_unseparated(format!("%{value}%"));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}

impl<T: Serialize + Default> FromRequestParts<AppState> for Filter<T> {
    type Rejection = FilterError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let Query(params): Query<HashMap<String, String>> =
            Query::from_request_parts(parts, state)
                .await
                .map_err(|_| FilterError::QueryError)?;

        // Serialize a default T to discover field names AND their JSON type.
        // Only string-typed fields (Value::String) are considered filterable -
        // this also covers Uuid, since it serializes as a string too.
        let default_value = serde_json::to_value(T::default())
            .map_err(|_| FilterError::SerdeError)?;

        let Value::Object(fields) = default_value else {
            return Err(FilterError::SerdeError);
        };

        let string_fields: Vec<String> = fields
            .into_iter()
            .filter_map(|(key, value)| matches!(value, Value::String(_)).then_some(key))
            .collect();

        let conditions = params
            .into_iter()
            .filter(|(key, _)| string_fields.contains(key))
            .collect();

        Ok(Filter { conditions, _marker: PhantomData })
    }
}

pub enum FilterError {
    QueryError,
	SerdeError,
}

impl IntoResponse for FilterError {
    fn into_response(self) -> Response {
        match self {
            FilterError::QueryError => (StatusCode::BAD_REQUEST, "Invalid query parameters"),
			FilterError::SerdeError => (StatusCode::INTERNAL_SERVER_ERROR, "Serde failed to generate default values")
        }
        .into_response()
    }
}