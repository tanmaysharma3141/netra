use uuid::Uuid;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::models::{Alert, Severity, UpdateAlertStatusRequest};
use crate::state::AppState;
use crate::stub_data;

#[derive(Debug, Deserialize)]
pub struct AlertQuery {
    pub case_id: Option<Uuid>,
    pub severity: Option<Severity>,
    pub status: Option<String>,
}

pub async fn list(Query(q): Query<AlertQuery>, State(_state): State<AppState>) -> Json<Vec<Alert>> {
    Json(
        stub_data::demo_alerts()
            .into_iter()
            .filter(|a| q.case_id.map_or(true, |c| a.case_id == c))
            .filter(|a| q.severity.map_or(true, |s| a.severity >= s))
            .collect(),
    )
}

pub async fn detail(Path(_id): Path<Uuid>) -> Json<Option<Alert>> {
    Json(stub_data::demo_alerts().into_iter().next())
}

pub async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAlertStatusRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!(alert = %id, status = ?req.status, note = ?req.note, "alert status change (stub)");
    state.publish("global", crate::models::WsEvent::TrainingProgress {
        payload: crate::models::TrainingProgress {
            epoch: 0,
            loss: 0.0,
            stage: format!("feedback-queued:{:?}", req.status),
        },
    });
    Ok(StatusCode::NO_CONTENT)
}
