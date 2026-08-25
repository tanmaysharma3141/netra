use uuid::Uuid;

use axum::extract::{Path, Query};
use axum::Json;
use serde::Deserialize;

use crate::models::TrailsResponse;
use crate::stub_data;

#[derive(Debug, Deserialize)]
pub struct MovementQuery {
    pub entity_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

pub async fn movements(
    Path(_case_id): Path<Uuid>,
    Query(q): Query<MovementQuery>,
) -> Json<TrailsResponse> {
    let mut trails = stub_data::demo_trails();
    if let Some(entity) = q.entity_id {
        trails.trails.retain(|t| t.entity_id == entity);
    }
    Json(trails)
}
