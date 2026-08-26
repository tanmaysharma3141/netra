use sqlx::FromRow;
use uuid::Uuid;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::auth::Authed;
use crate::db;
use crate::models::{Alert, ApiError, Role, Severity, UpdateAlertStatusRequest, WsEvent};
use crate::state::AppState;

#[derive(Debug, FromRow)]
struct AlertRow {
    id: String,
    case_id: String,
    pattern: String,
    severity: String,
    score: i64,
    status: String,
    entity_ids: String,
    evidence_event_ids: String,
    summary: String,
    created_at: String,
}

impl AlertRow {
    fn to_api(&self) -> Alert {
        Alert {
            id: Uuid::parse_str(&self.id).unwrap_or_default(),
            case_id: Uuid::parse_str(&self.case_id).unwrap_or_default(),
            pattern: self.pattern.clone(),
            severity: self.severity.parse().unwrap_or(Severity::Low),
            score: self.score as u8,
            status: self.status.parse().unwrap_or(crate::models::AlertStatus::Open),
            entity_ids: serde_json::from_str(&self.entity_ids)
                .unwrap_or_default(),
            evidence_event_ids: serde_json::from_str(&self.evidence_event_ids)
                .unwrap_or_default(),
            summary: self.summary.clone(),
            created_at: self.created_at.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AlertQuery {
    pub case_id: Option<Uuid>,
    pub severity: Option<Severity>,
    pub status: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    _authed: Authed,
    Query(q): Query<AlertQuery>,
) -> Result<Json<Vec<Alert>>, Response> {
    use sqlx::QueryBuilder;

    let mut qb = QueryBuilder::new(
        "SELECT id, case_id, pattern, severity, score, status, entity_ids, evidence_event_ids, summary, created_at FROM alerts WHERE 1=1 ",
    );
    if let Some(cid) = &q.case_id {
        qb.push(" AND case_id = ").push_bind(cid.to_string());
    }
    if let Some(sev) = &q.severity {
        qb.push(" AND severity = ").push_bind(sev.db_str());
    }
    if let Some(st) = &q.status {
        qb.push(" AND status = ").push_bind(st.clone());
    }
    qb.push(" ORDER BY created_at DESC LIMIT 500");

    let rows: Vec<AlertRow> = qb
        .build_query_as()
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

    Ok(Json(rows.iter().map(|r| r.to_api()).collect()))
}

pub async fn detail(
    State(state): State<AppState>,
    _authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<Alert>>, Response> {
    let row: Option<AlertRow> = sqlx::query_as(
        "SELECT id, case_id, pattern, severity, score, status, entity_ids, evidence_event_ids, summary, created_at FROM alerts WHERE id = ?1",
    )
    .bind(id.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;
    Ok(Json(row.map(|r| r.to_api())))
}

pub async fn update_status(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAlertStatusRequest>,
) -> Result<StatusCode, Response> {
    let res = sqlx::query("UPDATE alerts SET status = ?2, updated_at = ?3 WHERE id = ?1")
        .bind(id.to_string())
        .bind(serde_json::to_string(&req.status).unwrap_or_default().trim_matches('"').to_string())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.pool)
        .await
        .map_err(internal)?;
    if res.rows_affected() == 0 {
        return Err(ApiError::new("not_found", "alert not found").into_response(StatusCode::NOT_FOUND));
    }

    let case_id: Option<String> =
        sqlx::query_scalar("SELECT case_id FROM alerts WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&state.pool)
            .await
            .map_err(internal)?
            .flatten();

    db::audit(
        &state.pool,
        &authed.id,
        case_id.as_deref(),
        "alert.status_changed",
        serde_json::json!({ "alert_id": id.to_string(), "status": req.status, "note": req.note }),
    )
    .await;

    sqlx::query("INSERT INTO feedback_queue (id, kind, alert_id, label, note, user_id, created_at) VALUES (?1, 'alert_feedback', ?2, ?3, ?4, ?5, ?6)")
        .bind(Uuid::new_v4().to_string())
        .bind(id.to_string())
        .bind(serde_json::to_string(&req.status).unwrap_or_default().trim_matches('"').to_string())
        .bind(req.note.clone())
        .bind(authed.id.clone())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.pool)
        .await
        .map_err(internal)?;

    tracing::info!(alert = %id, status = ?req.status, "alert triaged");
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeBody {}

pub async fn analyze(
    State(state): State<AppState>,
    authed: Authed,
    Path(case_id): Path<Uuid>,
    Json(_body): Json<AnalyzeBody>,
) -> Result<Json<crate::anomaly::AnalyzeStats>, Response> {
    authed.require(&[Role::Admin, Role::Investigator, Role::Analyst])?;

    let _guard = state.pipeline_lock.lock().await;
    let pool = state.pool.clone();
    let stats = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(crate::anomaly::analyze_case(&pool, case_id))
    })
    .await
    .map_err(internal)?
    .map_err(internal)?;

    db::audit(
        &state.pool,
        &authed.id,
        Some(&case_id.to_string()),
        "analysis.run",
        serde_json::json!({ "alerts_raised": stats.alerts_raised }),
    )
    .await;

    publish_new_alerts(&state, case_id).await;

    Ok(Json(stats))
}

async fn publish_new_alerts(state: &AppState, case_id: Uuid) {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM alerts WHERE case_id = ?1 AND status = 'open' ORDER BY created_at DESC LIMIT 10",
    )
    .bind(case_id.to_string())
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    for (id,) in rows {
        if let Ok(aid) = Uuid::parse_str(&id) {
            if let Some(alert) = fetch_alert(&state.pool, aid).await {
                state.publish(format!("case:{case_id}"), WsEvent::AlertCreated { payload: alert });
            }
        }
    }
}

pub(crate) async fn fetch_alert(pool: &sqlx::SqlitePool, id: Uuid) -> Option<Alert> {
    let row: Option<AlertRow> = sqlx::query_as(
        "SELECT id, case_id, pattern, severity, score, status, entity_ids, evidence_event_ids, summary, created_at FROM alerts WHERE id = ?1",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.map(|r| r.to_api())
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error").into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
