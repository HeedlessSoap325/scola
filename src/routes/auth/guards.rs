use axum::{
    extract::{FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use tower_cookies::Cookies;
use uuid::Uuid;
use crate::common::{state::AppState, types::{Person, PersonRole}};

pub const SESSION_COOKIE: &str = "session_id";

pub struct AuthUser {
    pub id: Uuid,
    pub school_id: Uuid,
    pub email: String,
    pub login_name: String,
    pub first_name: String,
    pub last_name: String,
    pub picture: Option<String>,
    pub created_at: DateTime<Utc>,
    pub role: PersonRole,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // 1. Extract cookies from the request
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::MissingCookie)?;

        // 2. Decrypt + verify the private cookie
        let private = cookies.private(&state.cookie_key);
        let session_cookie = private
            .get(SESSION_COOKIE)
            .ok_or(AuthError::MissingCookie)?;

        let session_id = session_cookie.value().to_string();

        // 3. Look up the session
        let sessions = state.sessions.read().await;
        let user_id = sessions
            .get(&session_id)
            .cloned()
            .ok_or(AuthError::InvalidSession)?;

        // 4. Look up the user
        let person = sqlx::query_as::<_, Person>(
			r#"
				SELECT * FROM person p where p.id = $1 
			"#
		)
		.bind(user_id)
		.fetch_one(&state.pool)
		.await;

        match person {
			Ok(user) => Ok(AuthUser {
                id: user.id,
                school_id: user.school_id,
                email: user.email,
                login_name: user.login_name,
                first_name: user.first_name,
                last_name: user.last_name,
                picture: user.picture,
                created_at: user.created_at,
                role: user.role,
			}),
			Err(_) => Err(AuthError::InvalidSession),
		}
    }
}

/// Auth errors that produce proper HTTP responses
pub enum AuthError {
    MissingCookie,
    InvalidSession,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingCookie => (StatusCode::UNAUTHORIZED, "Not logged in"),
            AuthError::InvalidSession => (StatusCode::UNAUTHORIZED, "Session expired or invalid"),
        };
        (status, message).into_response()
    }
}