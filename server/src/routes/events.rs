use uuid::Uuid;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use sqlx::FromRow;

use crate::auth::Authed;
use crate::models::{Event, EntityType, EventType, LatLng, SourceType};
use crate::state::AppState;

#[derive(Debug, FromRow)]
pub struct EventRow {
    pub id: String,
    pub case_id: String,
    pub ts: String,
    pub source_type: String,
    pub entity_id: String,
    pub entity_type: String,
    pub event_type: String,
    pub value: Option<f64>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub raw: String,
    pub notes: String,
}

impl EventRow {
    fn to_api(&self) -> Event {
        Event {
            id: Uuid::parse_str(&self.id).unwrap_or_default(),
            case_id: Uuid::parse_str(&self.case_id).unwrap_or_default(),
            timestamp: chrono::DateTime::parse_from_rfc3339(&self.ts)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            source_type: self.source_type.parse().unwrap_or(SourceType::Cdr),
            entity_id: self.entity_id.clone(),
            entity_type: self.entity_type.parse().unwrap_or(EntityType::Phone),
            event_type: self.event_type.parse().unwrap_or(EventType::Other),
            value: self.value,
            location: match (self.lat, self.lng) {
                (Some(lat), Some(lng)) => Some(LatLng { lat, lng }),
                _ => None,
            },
            raw: serde_json::from_str(&self.raw).unwrap_or(serde_json::json!({})),
            notes: serde_json::from_str(&self.notes).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub source_type: Option<SourceType>,
    pub event_type: Option<EventType>,
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
    State(state): State<AppState>,
    _authed: Authed,
) -> Result<Json<Vec<Event>>, Response> {
    use sqlx::QueryBuilder;

    let mut qb = QueryBuilder::new(
        "SELECT id, case_id, ts, source_type, entity_id, entity_type, event_type, value, lat, lng, raw, notes FROM events WHERE \"case_id\" = ",
    );
    qb.push_bind(case_id.to_string());
    if let Some(st) = &q.source_type {
        qb.push(" AND source_type = ").push_bind(st.db_str());
    }
    if let Some(et) = &q.event_type {
        qb.push(" AND event_type = ").push_bind(et.db_str());
    }
    if let Some(eid) = &q.entity_id {
        qb.push(" AND entity_id = ").push_bind(eid.clone());
    }
    if let Some(from) = &q.from {
        if let Ok(f) = chrono::DateTime::parse_from_rfc3339(from) {
            qb.push(" AND ts >= ").push_bind(f.to_rfc3339());
        }
    }
    if let Some(to) = &q.to {
        if let Ok(t) = chrono::DateTime::parse_from_rfc3339(to) {
            qb.push(" AND ts <= ").push_bind(t.to_rfc3339());
        }
    }
    qb.push(" ORDER BY ts DESC LIMIT ")
        .push_bind((q.limit.min(1000) as i64).to_string())
        .push(" OFFSET ")
        .push_bind(q.offset as i64);

    let rows: Vec<EventRow> = qb
        .build_query_as()
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

    Ok(Json(rows.iter().map(|r| r.to_api()).collect()))
}

pub async fn detail(
    Path(_id): Path<Uuid>,
    State(state): State<AppState>,
    _authed: Authed,
) -> Result<Json<Option<Event>>, Response> {
    let row: Option<EventRow> = sqlx::query_as(
        "SELECT id, case_id, ts, source_type, entity_id, entity_type, event_type, value, lat, lng, raw, notes FROM events WHERE id = ?1",
    )
    .bind(_id.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;
    Ok(Json(row.map(|r| r.to_api())))
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    crate::models::ApiError::new("internal", "internal server error")
        .into_response(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}
