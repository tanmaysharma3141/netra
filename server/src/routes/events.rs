use uuid::Uuid;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::models::{Event, SourceType};
use crate::state::AppState;
use crate::stub_data;

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub source_type: Option<SourceType>,
    pub event_type: Option<String>,
    pub entity_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    200
}

pub async fn list(
    Path(case_id): Path<Uuid>,
    Query(q): Query<EventQuery>,
    State(_state): State<AppState>,
) -> Json<Vec<Event>> {
    let _ = case_id;
    let events: Vec<Event> = stub_data::demo_events()
        .into_iter()
        .filter(|e| q.source_type.map_or(true, |s| e.source_type == s))
        .filter(|e| q.entity_id.as_ref().map_or(true, |id| &e.entity_id == id))
        .skip(q.offset)
        .take(q.limit.min(1000))
        .collect();
    Json(events)
}

pub async fn detail(Path(_id): Path<Uuid>) -> Json<Option<Event>> {
    Json(stub_data::demo_events().into_iter().next())
}
