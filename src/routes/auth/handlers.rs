use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use bcrypt::verify;
use cookie::{time::Duration, Cookie, SameSite};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;
use crate::types::{Person, PersonRole, Student, Teacher};
use super::guards::{AuthUser, SESSION_COOKIE};
use super::models::*;

pub async fn login(State(state): State<AppState>, cookies: Cookies, Json(body): Json<LoginRequest>) -> Result<Json<AuthResponse>, AppError> {
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
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid credentials",
            )
        );
    }

    // Create session
    let session_id = Uuid::new_v4().to_string();

    state.sessions.write().await.insert(session_id.clone(), user_id.unwrap());

    // Build private (encrypted + signed) cookie
    let mut cookie = Cookie::new(SESSION_COOKIE, session_id);
    cookie.set_http_only(true);
    cookie.set_secure(false);
    cookie.set_same_site(SameSite::Strict); // CSRF protection
    cookie.set_path("/");
    cookie.set_max_age(Duration::hours(24));

    cookies.private(&state.cookie_key).add(cookie);

    Ok(Json(AuthResponse { message: "Logged in".into() }))
}

pub async fn logout(State(state): State<AppState>, cookies: Cookies, _: AuthUser) -> Json<AuthResponse> {
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

    Json(AuthResponse { message: "Logged out".into() })
}

pub async fn me(State(state): State<AppState>, auth: AuthUser) -> Result<Json<MeResponse>, AppError> {
    let (first_name, last_name, picture) = match auth.role {
        PersonRole::Student => {
            let s = sqlx::query_as::<_, Student>("SELECT * FROM student WHERE id = $1")
                .bind(auth.id)
                .fetch_one(&state.pool)
                .await
                .map_err(|_| AppError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Person is a student, but student record wasn't found!",
                ))?;
            (Some(s.first_name), Some(s.last_name), Some(s.picture))
        }
        PersonRole::Teacher => {
            let t = sqlx::query_as::<_, Teacher>("SELECT * FROM teacher WHERE id = $1")
                .bind(auth.id)
                .fetch_one(&state.pool)
                .await
                .map_err(|_| AppError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Person is a teacher, but teacher record wasn't found!",
                ))?;
            (Some(t.first_name), Some(t.last_name), None)
        }
        _ => (None, None, None),
    };

    Ok(Json(MeResponse {
        id: auth.id,
        email: auth.email,
        login_name: auth.login_name,
        first_name,
        last_name,
        picture,
    }))
}