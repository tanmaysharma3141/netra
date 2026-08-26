use uuid::Uuid;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::auth::Authed;
use crate::models::{ApiError, TrailsResponse};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct MovementQuery {
    pub entity_id: Option<String>,
    #[allow(dead_code)]
    pub from: Option<String>,
    #[allow(dead_code)]
    pub to: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct LocationRow {
    entity_id: String,
    lat: f64,
    lng: f64,
    tower_id: Option<String>,
    ts: String,
}

pub async fn movements(
    State(state): State<AppState>,
    _authed: Authed,
    Path(case_id): Path<Uuid>,
    Query(q): Query<MovementQuery>,
) -> Result<Json<TrailsResponse>, Response> {
    let cid = case_id.to_string();

    // Check case exists
    let case_exists: Option<String> = sqlx::query_scalar("SELECT id FROM cases WHERE id = ?1")
        .bind(&cid)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;
    if case_exists.is_none() {
        return Err(
            ApiError::new("not_found", "case not found")
                .into_response(StatusCode::NOT_FOUND),
        );
    }

    // Build query — get events that have location data (lat/lng)
    use sqlx::QueryBuilder;
    let mut qb = QueryBuilder::new(
        "SELECT entity_id, lat, lng, \
         json_extract(raw, '$.cell_id') as tower_id, ts \
         FROM events WHERE case_id = ",
    );
    qb.push_bind(&cid);
    qb.push(" AND lat IS NOT NULL AND lng IS NOT NULL");

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
    qb.push(" ORDER BY ts ASC LIMIT 5000");

    let rows: Vec<LocationRow> = qb
        .build_query_as()
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

    // Group by entity_id into trails
    let mut trails_map: std::collections::HashMap<String, Vec<crate::models::GeoPoint>> =
        std::collections::HashMap::new();
    for row in rows {
        let point = crate::models::GeoPoint {
            entity_id: row.entity_id.clone(),
            lat: row.lat,
            lng: row.lng,
            tower_id: row.tower_id,
            timestamp: chrono::DateTime::parse_from_rfc3339(&row.ts)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        };
        trails_map.entry(row.entity_id).or_default().push(point);
    }

    let trails = trails_map
        .into_iter()
        .map(|(entity_id, points)| crate::models::Trail { entity_id, points })
        .collect();

    Ok(Json(TrailsResponse { trails }))
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
