use axum::extract::FromRequestParts;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::models::{ApiError, Role};
use crate::state::AppState;

const LOCKOUT_THRESHOLD: i64 = 5;
const LOCKOUT_MINUTES: i64 = 15;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub iat: usize,
    pub exp: usize,
    pub jti: String,
}

pub fn issue_token(user_id: &str, role: &str, secret: &str) -> (String, chrono::DateTime<Utc>) {
    let now = Utc::now();
    let expires_at = now + Duration::hours(8);
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: now.timestamp() as usize,
        exp: expires_at.timestamp() as usize,
        jti: uuid::Uuid::new_v4().to_string(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("jwt encoding cannot fail with valid key");
    (token, expires_at)
}

pub fn verify_token(token: &str, secret: &str) -> Option<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims)
}

fn unauthorized(message: &str) -> Response {
    ApiError::new("unauthorized", message).into_response(StatusCode::UNAUTHORIZED)
}

pub struct Authed {
    pub id: String,
    pub username: String,
    pub role: Role,
}

impl FromRequestParts<AppState> for Authed {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let bearer = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_string);

        let token = match bearer {
            Some(t) => t,
            None => parts
                .uri
                .query()
                .and_then(|q| {
                    q.split('&').find_map(|pair| {
                        pair.strip_prefix("token=").map(str::to_string)
                    })
                })
                .ok_or_else(|| unauthorized("missing bearer token"))?,
        };

        let claims =
            verify_token(&token, &state.jwt_secret).ok_or_else(|| unauthorized("invalid token"))?;

        let active_row: Option<i64> =
            sqlx::query_scalar("SELECT active FROM users WHERE id = ?1")
                .bind(&claims.sub)
                .fetch_optional(&state.pool)
                .await
                .ok()
                .flatten();
        match active_row {
            Some(active) if active != 0 => {}
            _ => return Err(unauthorized("user inactive or removed")),
        }

        let username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = ?1")
            .bind(&claims.sub)
            .fetch_one(&state.pool)
            .await
            .unwrap_or_default();

        Ok(Authed {
            id: claims.sub,
            username,
            role: claims.role.parse().map_err(|_| unauthorized("bad role claim"))?,
        })
    }
}

impl Authed {
    pub fn require(&self, allowed: &[Role]) -> Result<(), Response> {
        if allowed.contains(&self.role) {
            Ok(())
        } else {
            Err(ApiError::new(
                "forbidden",
                format!("requires one of: {:?}", allowed),
            )
            .into_response(StatusCode::FORBIDDEN))
        }
    }
}

pub struct LoginOutcome {
    pub ok: bool,
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

pub async fn attempt_login(
    pool: &sqlx::SqlitePool,
    username: &str,
    password: &str,
) -> Result<db::UserRow, LoginOutcome> {
    let row: Option<db::UserRow> =
        sqlx::query_as("SELECT * FROM users WHERE username = ?1 COLLATE NOCASE")
            .bind(username)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    let Some(user) = row else {
        return Err(LoginOutcome {
            ok: false,
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "invalid credentials".into(),
        });
    };

    if user.active == 0 {
        return Err(LoginOutcome {
            ok: false,
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "account deactivated".into(),
        });
    }

    if let Some(locked_str) = &user.locked_until {
        if let Ok(locked_until) = chrono::DateTime::parse_from_rfc3339(locked_str) {
            let locked_utc = locked_until.with_timezone(&chrono::Utc);
            if locked_utc > chrono::Utc::now() {
                let retry_secs = (locked_utc - chrono::Utc::now()).num_seconds().max(1);
                return Err(LoginOutcome {
                    ok: false,
                    status: StatusCode::LOCKED,
                    code: "locked",
                    message: format!("account locked; retry in {retry_secs}s"),
                });
            }
        }
    }

    match bcrypt::verify(password, &user.password_hash) {
        Ok(true) => {
            let _ = sqlx::query("UPDATE users SET failed_attempts = 0, locked_until = NULL WHERE id = ?1")
                .bind(&user.id)
                .execute(pool)
                .await;
            Ok(user)
        }
        Ok(false) => {
            let attempts = user.failed_attempts + 1;
            if attempts >= LOCKOUT_THRESHOLD {
                let until = (chrono::Utc::now() + Duration::minutes(LOCKOUT_MINUTES)).to_rfc3339();
                let _ = sqlx::query("UPDATE users SET failed_attempts = ?2, locked_until = ?3 WHERE id = ?1")
                    .bind(&user.id)
                    .bind(attempts)
                    .bind(until)
                    .execute(pool)
                    .await;
                Err(LoginOutcome {
                    ok: false,
                    status: StatusCode::UNAUTHORIZED,
                    code: "unauthorized",
                    message: "invalid credentials; account locked".into(),
                })
            } else {
                let _ = sqlx::query("UPDATE users SET failed_attempts = ?2 WHERE id = ?1")
                    .bind(&user.id)
                    .bind(attempts)
                    .execute(pool)
                    .await;
                Err(LoginOutcome {
                    ok: false,
                    status: StatusCode::UNAUTHORIZED,
                    code: "unauthorized",
                    message: "invalid credentials".into(),
                })
            }
        }
        Err(_) => Err(LoginOutcome {
            ok: false,
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal",
            message: "password verification failed".into(),
        }),
    }
}
