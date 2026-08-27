use axum::extract::State;
use axum::Json;
use std::time::Instant;

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub uptime_seconds: u64,
    pub db_size_mb: f64,
    pub events_count: u64,
    pub entities_count: u64,
    pub alerts_count: u64,
    pub cases_count: u64,
}

static START_TIME: once_cell::sync::Lazy<Instant> = once_cell::sync::Lazy::new(Instant::now);

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let uptime = START_TIME.elapsed().as_secs();

    let db_size_mb = std::fs::metadata("netra.db")
        .map(|m| m.len() as f64 / (1024.0 * 1024.0))
        .unwrap_or(0.0);

    let events_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let entities_count: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT id) FROM entities")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let alerts_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alerts")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let cases_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cases")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: uptime,
        db_size_mb,
        events_count: events_count.0 as u64,
        entities_count: entities_count.0 as u64,
        alerts_count: alerts_count.0 as u64,
        cases_count: cases_count.0 as u64,
    })
}
