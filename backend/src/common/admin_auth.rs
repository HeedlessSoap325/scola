use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{common::{error::{AppError, db_error}, types::PersonRole}, routes::auth::guards::AuthUser};

/// Call this at the top of any handler that needs a school-scoped operation.
/// Returns the school_id to use, or an error.
///
/// - LocalAdmin: returns their own school_id (ignores body.school_id)
/// - Admin: validates and returns body.school_id
/// - Other: doesn't validate and returns an Error
pub async fn resolve_school(
    user: &AuthUser,
    requested_school_id: Option<Uuid>,
    pool: &PgPool,
) -> Result<Uuid, AppError> {
    match user.role {
        PersonRole::LocalAdmin => {
            Ok(user.school_id)
        }
        PersonRole::Admin => {
            let school_id = requested_school_id.ok_or(AppError(
                StatusCode::BAD_REQUEST,
                "school_id is required for admin operations",
            ))?;

            // Verify the school actually exists
            let exists = sqlx::query_scalar!(
                "SELECT EXISTS ( SELECT 1 FROM school WHERE id = $1 )",
                school_id
            )
            .fetch_one(pool)
            .await
            .map_err(db_error)?
            .unwrap_or(false);

            if !exists {
                return Err(AppError(StatusCode::NOT_FOUND, "School not found"));
            }

            Ok(school_id)
        }
        _ => Err(AppError(StatusCode::UNAUTHORIZED, "Insufficient privileges")),
    }
}

/// Call this at the top of any handler that is a admin operation without school-based operations.
/// Returns an AppError, if the user is unprivileged.
///
/// - LocalAdmin: ok
/// - Admin: ok
/// - Other: Error
pub fn is_admin(
    user: &AuthUser
) -> Result<(), AppError>
{
    if user.role != PersonRole::Admin && user.role != PersonRole::LocalAdmin {
		return Err(
			AppError(StatusCode::UNAUTHORIZED, "Insufficient privileges")
		);
	}
    Ok(())
}