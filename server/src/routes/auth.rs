use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{Duration, Utc};

use crate::models::{LoginRequest, LoginResponse};
use crate::state::AppState;
use crate::stub_data;

pub async fn login(
    State(_state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user = stub_data::admin_user();
    tracing::info!(username = %req.username, "login (stub)");
    Ok(Json(LoginResponse {
        token: format!("stub-token-{}", AppState::new_id()),
        expires_at: Utc::now() + Duration::hours(8),
        user,
    }))
}

pub async fn logout() -> StatusCode {
    StatusCode::NO_CONTENT
}
