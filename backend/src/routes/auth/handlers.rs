use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use bcrypt::verify;
use cookie::{time::Duration, Cookie, SameSite};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::session::write_session;
use crate::common::state::{AppState, Session};
use crate::common::types::{GenericResponse, Person};
use super::guards::{AuthUser, SESSION_COOKIE};
use super::models::*;

pub async fn login(State(state): State<AppState>, cookies: Cookies, Json(body): Json<LoginRequest>) -> Result<GenericResponse, AppError> {
    let person = sqlx::query_as::<_, Person>(
		r#"
			SELECT * FROM person p  where p.login_name = $1 and p.school_id = $2
		"#
	)
	.bind(&body.login_name)
    .bind(body.school)
	.fetch_one(&state.pool)
	.await;

    // Look up user — always verify hash even on miss to prevent timing attacks
    let dummy_hash = "$2b$12$invalidhashfortimingprotection000000000000000000000000000";
    let (user_id, stored_hash) = match person {
        Ok(user) => (Some(user.id), user.password),
        Err(_) => (None, dummy_hash.to_string()),
    };

    let password_valid = verify(&body.password, &stored_hash).unwrap_or(false);

    if !password_valid || user_id.is_none() {
        return Err(
            AppError(
                StatusCode::FORBIDDEN,
                "Invalid credentials",
            )
        );
    }

    // Create session
    let session_id = write_session(state.redis, user_id.unwrap()).await?;

    // Build private (encrypted + signed) cookie
    let mut cookie = Cookie::new(SESSION_COOKIE, session_id);
    cookie.set_http_only(true);
    cookie.set_secure(false);
    cookie.set_same_site(SameSite::Strict); // CSRF protection
    cookie.set_path("/");
    cookie.set_max_age(Duration::hours(24));

    cookies.private(&state.cookie_key).add(cookie);

    Ok(GenericResponse(StatusCode::OK, "Logged in"))
}

pub async fn logout(State(state): State<AppState>, cookies: Cookies, _: AuthUser) -> GenericResponse {
    let private = cookies.private(&state.cookie_key);

    if let Some(session_cookie) = private.get(SESSION_COOKIE) {
        let session_id = session_cookie.value().to_string();

        // Invalidate server-side session
        state.sessions.write().await.remove(&session_id);

        // Remove the cookie from the client
        let mut removal = Cookie::from(SESSION_COOKIE);
        removal.set_path("/");
        cookies.remove(removal);
    }

    GenericResponse(StatusCode::OK, "Logged out")
}

pub async fn logout_all(
    State(state): State<AppState>,
    cookies: Cookies,
    user: AuthUser,
) -> Result<GenericResponse, AppError>
{
    // Invalidate server-side sessions
    state.sessions.write().await.retain(|_, session| session.user_id != user.id);
      
    // Remove the cookie from the client
    let mut removal = Cookie::from(SESSION_COOKIE);
    removal.set_path("/");
    cookies.remove(removal);

    Ok(GenericResponse(StatusCode::OK, "Logged out"))
}

pub async fn me(auth: AuthUser) -> Result<Json<MeResponse>, AppError> {
    Ok(Json(MeResponse {
        id: auth.id,
        email: auth.email,
        login_name: auth.login_name,
        first_name: auth.first_name,
        last_name: auth.last_name,
        picture: auth.picture,
    }))
}