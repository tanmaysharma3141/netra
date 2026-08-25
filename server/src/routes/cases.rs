use uuid::Uuid;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::models::{AuditEntry, Case, CreateCaseRequest};
use crate::state::AppState;
use crate::stub_data;

pub async fn list(State(state): State<AppState>) -> Json<Vec<Case>> {
    state
        .request_counter
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Json(vec![stub_data::demo_case()])
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateCaseRequest>,
) -> Json<Case> {
    state
        .request_counter
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut demo = stub_data::demo_case();
    demo.id = AppState::new_id();
    demo.title = req.title;
    demo.classification = if req.classification.is_empty() {
        "UNCLASSIFIED".into()
    } else {
        req.classification
    };
    demo.tags = req.tags;
    demo.stats = Default::default();
    tracing::info!(case_id = %demo.id, "case created (stub)");
    Json(demo)
}

pub async fn detail(Path(id): Path<Uuid>) -> Result<Json<Case>, (StatusCode, String)> {
    if id == stub_data::CASE_ID {
        Ok(Json(stub_data::demo_case()))
    } else {
        Err((StatusCode::NOT_FOUND, "case not found (stub has one demo case)".into()))
    }
}

pub async fn update(Path(_id): Path<Uuid>) -> StatusCode {
    StatusCode::NO_CONTENT
}

pub async fn audit(Path(id): Path<Uuid>) -> Json<Vec<AuditEntry>> {
    let _ = id;
    Json(stub_data::demo_audit())
}
