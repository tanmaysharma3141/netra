use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::models::{CreateUserRequest, User};
use crate::state::AppState;
use crate::stub_data;

pub async fn list(State(state): State<AppState>) -> Json<Vec<User>> {
    state
        .request_counter
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Json(vec![stub_data::admin_user()])
}

pub async fn create(
    State(_state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Json<User> {
    Json(User {
        id: AppState::new_id(),
        username: req.username,
        role: req.role,
        active: true,
    })
}

pub async fn update() -> StatusCode {
    StatusCode::NO_CONTENT
}

pub async fn deactivate() -> StatusCode {
    StatusCode::NO_CONTENT
}
