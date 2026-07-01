use serde::Serialize;
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::common::error::{AppError, db_error};

fn get_struct_name<T>() -> String {
    let full_name = std::any::type_name::<T>(); // "my_crate::models::User"
    let short_name = full_name.rsplit("::").next().unwrap_or(full_name);
    let mut chars = short_name.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
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