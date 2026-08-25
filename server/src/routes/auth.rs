use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::auth;
use crate::db;
use crate::models::{ApiError, LoginRequest, LoginResponse};
use crate::state::AppState;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, axum::response::Response> {
    let outcome = auth::attempt_login(&state.pool, &req.username, &req.password).await;

    match outcome {
        Ok(user) => {
            let (token, expires_at) = auth::issue_token(&user.id, &user.role, &state.jwt_secret);
            db::audit(
                &state.pool,
                &user.id,
                None,
                "auth.login",
                serde_json::json!({ "username": req.username }),
            )
            .await;
            tracing::info!(username = %req.username, "login success");
            Ok(Json(LoginResponse {
                token,
                expires_at,
                user: user.to_api(),
            }))
        }
        Err(outcome) => {
            if outcome.status == StatusCode::UNAUTHORIZED || outcome.status == StatusCode::LOCKED {
                tracing::warn!(username = %req.username, code = %outcome.code, "login failed");
            }
            Err(ApiError::new(outcome.code, outcome.message)
                .into_response(outcome.status))
        }
    }
}

pub async fn logout(
    State(state): State<AppState>,
    authed: crate::auth::Authed,
) -> StatusCode {
    db::audit(
        &state.pool,
        &authed.id,
        None,
        "auth.logout",
        serde_json::json!({}),
    )
    .await;
    StatusCode::NO_CONTENT
}
