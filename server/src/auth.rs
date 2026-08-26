use axum::extract::FromRequestParts;
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

/// Check if a token's jti has been revoked.
pub async fn is_token_revoked(pool: &sqlx::SqlitePool, jti: &str) -> bool {
    sqlx::query_scalar::<_, i64>("SELECT 1 FROM revoked_tokens WHERE jti = ?1 LIMIT 1")
        .bind(jti)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .is_some()
}

/// Revoke a token by inserting its jti into the revoked_tokens table.
pub async fn revoke_token(pool: &sqlx::SqlitePool, jti: &str) {
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO revoked_tokens (jti, revoked_at) VALUES (?1, ?2)",
    )
    .bind(jti)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await;
}

/// Cleanup expired revoked tokens (older than 9 hours).
pub async fn cleanup_revoked_tokens(pool: &sqlx::SqlitePool) {
    let cutoff = (chrono::Utc::now() - chrono::Duration::hours(9)).to_rfc3339();
    let _ = sqlx::query("DELETE FROM revoked_tokens WHERE revoked_at < ?1")
        .bind(cutoff)
        .execute(pool)
        .await;
}

fn unauthorized(message: &str) -> Response {
    ApiError::new("unauthorized", message).into_response(StatusCode::UNAUTHORIZED)
}

pub(crate) fn unauthorized_response(message: &str) -> Response {
    unauthorized(message)
}

pub struct Authed {
    pub id: String,
    #[allow(dead_code)]
    pub username: String,
    pub role: Role,
    /// The token's jti claim — used for revocation on logout
    pub jti: String,
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

        // Check if token has been revoked (logout)
        if is_token_revoked(&state.pool, &claims.jti).await {
            return Err(unauthorized("token revoked"));
        }

        let row: Option<(String, String, i64)> =
            sqlx::query_as("SELECT username, role, active FROM users WHERE id = ?1")
                .bind(&claims.sub)
                .fetch_optional(&state.pool)
                .await
                .map_err(|_| unauthorized("lookup failed"))?;

        let Some((username, role, active)) = row else {
            return Err(unauthorized("user removed"));
        };
        if active == 0 {
            return Err(unauthorized("user inactive"));
        }

        Ok(Authed {
            id: claims.sub,
            username,
            role: role.parse().map_err(|_| unauthorized("bad role"))?,
            jti: claims.jti,
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
    #[allow(dead_code)]
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
