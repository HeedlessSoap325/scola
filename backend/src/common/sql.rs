use std::{collections::HashMap};

use axum::http::StatusCode;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::Serialize;
use serde_json::{Map, Value};
use sqlx::{PgPool, Postgres, QueryBuilder, query_builder::Separated};
use uuid::Uuid;

use crate::common::error::{AppError, db_error};

fn get_field_map<T: Serialize>(value: &T) -> Map<String, Value>{
    let json = serde_json::to_value(value).unwrap();
    if let Value::Object(map) = json {
        map
    } else {
		Map::new()
	}
}

async fn get_column_types(
    pool: &PgPool,
    table_name: &str,
) -> Result<HashMap<String, String>, AppError> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT column_name, udt_name FROM information_schema.columns WHERE table_name = $1",
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .map_err(db_error)?;

    Ok(rows.into_iter().collect())
}

fn push_typed_bind(
    sep: &mut Separated<'_, Postgres, &str>,
    value: Value,
    udt_name: &str,
) -> Result<(), AppError> {
    let type_err = |_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "value doesn't match column type");

    match udt_name {
        "uuid" => {
            let v: Uuid = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "text" | "varchar" | "bpchar" | "citext" => {
            let v: String = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "int2" => {
            let v: i16 = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "int4" => {
            let v: i32 = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "int8" => {
            let v: i64 = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "float4" => {
            let v: f32 = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "float8" => {
            let v: f64 = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "bool" => {
            let v: bool = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "timestamp" => {
            let v: NaiveDateTime = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "timestamptz" => {
            let v: DateTime<Utc> = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "date" => {
            let v: NaiveDate = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        }
        "jsonb" | "json" => {
            sep.push_bind(sqlx::types::Json(value));
        },
        "numeric" => {
            let v: BigDecimal = serde_json::from_value(value).map_err(type_err)?;
            sep.push_bind(v);
        },
        other => {
			println!("unsupported column type: {}", other);
            return Err(AppError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unsupported column type",
            ));
        }
    }
    Ok(())
}

fn get_struct_name<T>() -> String {
    let full_name = std::any::type_name::<T>(); // "my_crate::models::User"
    let short_name = full_name.rsplit("::").next().unwrap_or(full_name);
    let mut chars = short_name.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Create a resource of type T.
/// 
/// The name of the generic "T" will be used as the tablename (first letter will not be capital)
/// So the Struct for "T" must be the same name as the table to create
/// The resource passed must have all the fields set.
pub async fn create_resource<T: Serialize>(
    pool: &PgPool,
    resource: T,
) -> Result<(), AppError> {
    let table_name = get_struct_name::<T>();
    let field_map = get_field_map::<T>(&resource);

    if field_map.is_empty() {
        return Ok(());
    }

    let column_types = get_column_types(pool, &table_name).await?;
    if column_types.is_empty() {
        return Err(AppError(StatusCode::BAD_REQUEST, "unknown table"));
    }

    let mut builder = QueryBuilder::new(format!("INSERT INTO {}(", quote_ident(&table_name)));
    let mut sep_cols = builder.separated(", ");
    for name in field_map.keys() {
        if !column_types.contains_key(name) {
            return Err(AppError(StatusCode::BAD_REQUEST, "unknown column"));
        }
        sep_cols.push(quote_ident(name));
    }

    builder.push(") VALUES (");
    let mut sep_vals = builder.separated(", ");
    for (name, value) in field_map {
        let udt_name = &column_types[&name];
        push_typed_bind(&mut sep_vals, value, udt_name)?;
    }
    builder.push(")");

    builder.build().execute(pool).await.map_err(db_error)?;
    Ok(())
}

/// Delete a resource of type T, identified by it's ID.
/// 
/// The name of the generic "T" will be used as the tablename (first letter will not be capital)
/// So the Struct for "T" must be the same name as the table to delete
pub async fn delete_resource<T: Serialize>(
	pool: &PgPool, 
	id: Uuid,
) -> Result<(), AppError>
{
	let table_name = get_struct_name::<T>();

	let mut builder = QueryBuilder::new(format!("DELETE FROM {} WHERE id = ", quote_ident(&table_name)));
	builder.push_bind(id);

	builder
		.build()
		.execute(pool)
		.await
		.map_err(db_error)?;

	Ok(())
}

/// Wraps an identifier in double quotes and escapes embedded quotes,
/// as defense in depth on top of the allow-list check.
fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}