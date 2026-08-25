use uuid::Uuid;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::auth::Authed;
use crate::db;
use crate::models::{ApiError, CreateUserRequest, Role, User};
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    authed: Authed,
) -> Result<Json<Vec<User>>, Response> {
    authed.require(&[Role::Admin])?;
    let rows: Vec<db::UserRow> = sqlx::query_as("SELECT * FROM users ORDER BY created_at")
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;
    Ok(Json(rows.iter().map(|r| r.to_api()).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    authed: Authed,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, Response> {
    authed.require(&[Role::Admin])?;
    if req.username.trim().is_empty() || req.password.len() < 8 {
        return Err(ApiError::new("bad_request", "username required; password must be >= 8 chars")
            .into_response(StatusCode::BAD_REQUEST));
    }
    let hash = bcrypt::hash(&req.password, 12).map_err(|_| internal("bcrypt failure"))?;
    let id = Uuid::new_v4();
    let role = serde_json::to_string(&req.role).unwrap_or_else(|_| "\"analyst\"".into());
    let role = role.trim_matches('"').to_string();
    let result = sqlx::query("INSERT INTO users (id, username, password_hash, role, active, failed_attempts, created_at) VALUES (?1, ?2, ?3, ?4, 1, 0, ?5)")
        .bind(id.to_string())
        .bind(req.username.trim())
        .bind(hash)
        .bind(&role)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => {
            db::audit(
                &state.pool,
                &authed.id,
                None,
                "user.created",
                serde_json::json!({ "new_user": req.username, "role": role }),
            )
            .await;
            Ok(Json(User {
                id,
                username: req.username.trim().to_string(),
                role: req.role,
                active: true,
            }))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err(ApiError::new(
            "conflict",
            "username already exists",
        )
        .into_response(StatusCode::CONFLICT)),
        Err(e) => Err(internal(format!("db insert failed: {e}"))),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<Role>,
    pub password: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, Response> {
    authed.require(&[Role::Admin])?;

    if let Some(role) = &req.role {
        let role_str = serde_json::to_string(role).unwrap_or_default();
        let role_str = role_str.trim_matches('"');
        sqlx::query("UPDATE users SET role = ?2 WHERE id = ?1")
            .bind(id.to_string())
            .bind(role_str)
            .execute(&state.pool)
            .await
            .map_err(internal)?;
    }
    if let Some(password) = &req.password {
        if password.len() < 8 {
            return Err(ApiError::new("bad_request", "password must be >= 8 chars")
                .into_response(StatusCode::BAD_REQUEST));
        }
        let hash = bcrypt::hash(password, 12).map_err(|_| internal("bcrypt failure"))?;
        sqlx::query("UPDATE users SET password_hash = ?2, failed_attempts = 0, locked_until = NULL WHERE id = ?1")
            .bind(id.to_string())
            .bind(hash)
            .execute(&state.pool)
            .await
            .map_err(internal)?;
    }

    db::audit(
        &state.pool,
        &authed.id,
        None,
        "user.updated",
        serde_json::json!({ "target": id.to_string(), "role_reset": req.role.is_some(), "password_reset": req.password.is_some() }),
    )
    .await;

    fetch_user(&state.pool, id)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::new("not_found", "user not found").into_response(StatusCode::NOT_FOUND))
}

pub async fn deactivate(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    authed.require(&[Role::Admin])?;
    if id.to_string() == authed.id {
        return Err(ApiError::new("bad_request", "cannot deactivate yourself")
            .into_response(StatusCode::BAD_REQUEST));
    }
    let res = sqlx::query("UPDATE users SET active = 0 WHERE id = ?1")
        .bind(id.to_string())
        .execute(&state.pool)
        .await
        .map_err(internal)?;
    if res.rows_affected() == 0 {
        return Err(ApiError::new("not_found", "user not found").into_response(StatusCode::NOT_FOUND));
    }
    db::audit(
        &state.pool,
        &authed.id,
        None,
        "user.deactivated",
        serde_json::json!({ "target": id.to_string() }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn fetch_user(pool: &sqlx::SqlitePool, id: Uuid) -> Result<Option<User>, Response> {
    let row: Option<db::UserRow> = sqlx::query_as("SELECT * FROM users WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(internal)?;
    Ok(row.map(|r| r.to_api()))
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error").into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
