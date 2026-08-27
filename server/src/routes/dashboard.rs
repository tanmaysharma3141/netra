use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;

use crate::auth::Authed;
use crate::models::{ApiError, Alert, Severity};
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct DashboardStats {
    pub total_cases: u64,
    pub active_cases: u64,
    pub alerts_by_severity: SeverityCounts,
    pub recent_alerts: Vec<Alert>,
    pub events_this_week: u64,
    pub entities_count: u64,
}

#[derive(serde::Serialize)]
pub struct SeverityCounts {
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
}

pub async fn dashboard(
    State(state): State<AppState>,
    _authed: Authed,
) -> Result<Json<DashboardStats>, Response> {
    // Total cases (role-scoped)
    let total_cases: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cases")
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;

    let active_cases: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cases WHERE status = 'active'")
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;

    // Alert counts by severity
    let crit: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alerts WHERE severity = 'critical' AND status = 'open'")
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;
    let high: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alerts WHERE severity = 'high' AND status = 'open'")
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;
    let med: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alerts WHERE severity = 'medium' AND status = 'open'")
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;
    let low: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alerts WHERE severity = 'low' AND status = 'open'")
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;

    // Recent alerts (last 10)
    let recent_alert_rows: Vec<(String, String, String, String, i64, String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, case_id, pattern, severity, score, status, entity_ids, evidence_event_ids, summary, created_at, updated_at FROM alerts ORDER BY created_at DESC LIMIT 10",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;

    let recent_alerts: Vec<Alert> = recent_alert_rows.into_iter().filter_map(|row| {
        let id = uuid::Uuid::parse_str(&row.0).ok()?;
        let case_id = uuid::Uuid::parse_str(&row.1).ok()?;
        Some(Alert {
            id,
            case_id,
            pattern: row.2,
            severity: row.3.parse().unwrap_or(Severity::Medium),
            score: row.4 as u8,
            status: row.5.parse().unwrap_or(crate::models::AlertStatus::Open),
            entity_ids: serde_json::from_str(&row.6).unwrap_or_default(),
            evidence_event_ids: serde_json::from_str(&row.7).unwrap_or_default(),
            summary: row.8,
            created_at: row.9,
        })
    }).collect();

    // Events this week
    let week_ago = (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339();
    let events_this_week: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE ts >= ?1")
        .bind(&week_ago)
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;

    // Total entities
    let entities_count: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT id) FROM entities")
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;

    Ok(Json(DashboardStats {
        total_cases: total_cases.0 as u64,
        active_cases: active_cases.0 as u64,
        alerts_by_severity: SeverityCounts {
            critical: crit.0 as u64,
            high: high.0 as u64,
            medium: med.0 as u64,
            low: low.0 as u64,
        },
        recent_alerts,
        events_this_week: events_this_week.0 as u64,
        entities_count: entities_count.0 as u64,
    }))
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
