use uuid::Uuid;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::auth::Authed;
use crate::db;
use crate::models::{ApiError, AuditEntry, Case, CaseStats, CreateCaseRequest, Role};
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    authed: Authed,
) -> Result<Json<Vec<Case>>, Response> {
    let rows: Vec<db::CaseRow> = sqlx::query_as("SELECT * FROM cases ORDER BY created_at DESC")
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;

    let visible: Vec<db::CaseRow> = match authed.role {
        Role::Admin | Role::Supervisor => rows,
        _ => rows
            .into_iter()
            .filter(|c| {
                c.created_by == authed.id || c.assignee_ids().iter().any(|a| a == &authed.id)
            })
            .collect(),
    };

    let mut cases: Vec<Case> = visible.iter().map(|r| r.to_api()).collect();
    fill_stats(&state, &mut cases).await;
    Ok(Json(cases))
}

pub async fn create(
    State(state): State<AppState>,
    authed: Authed,
    Json(req): Json<CreateCaseRequest>,
) -> Result<Json<Case>, Response> {
    authed.require(&[Role::Admin, Role::Investigator])?;
    if req.title.trim().is_empty() {
        return Err(ApiError::new("bad_request", "title is required")
            .into_response(StatusCode::BAD_REQUEST));
    }
    let id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    let assignees = serde_json::json!([authed.id]).to_string();
    let tags = serde_json::to_string(&req.tags).unwrap_or_else(|_| "[]".into());
    sqlx::query("INSERT INTO cases (id, title, status, classification, created_by, created_at, tags, assignees) VALUES (?1, ?2, 'active', ?3, ?4, ?5, ?6, ?7)")
        .bind(id.to_string())
        .bind(req.title.trim())
        .bind(if req.classification.is_empty() { "UNCLASSIFIED" } else { req.classification.as_str() })
        .bind(authed.id.clone())
        .bind(now)
        .bind(tags)
        .bind(assignees)
        .execute(&state.pool)
        .await
        .map_err(internal)?;

    db::audit(
        &state.pool,
        &authed.id,
        Some(&id.to_string()),
        "case.created",
        serde_json::json!({ "title": req.title }),
    )
    .await;

    Ok(Json(Case {
        id,
        title: req.title.trim().to_string(),
        status: crate::models::CaseStatus::Active,
        classification: if req.classification.is_empty() {
            "UNCLASSIFIED".into()
        } else {
            req.classification
        },
        created_by: Uuid::parse_str(&authed.id).unwrap_or_default(),
        created_at: chrono::Utc::now(),
        assignees: vec![Uuid::parse_str(&authed.id).unwrap_or_default()],
        tags: req.tags,
        stats: CaseStats::default(),
    }))
}

pub async fn detail(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<Json<Case>, Response> {
    let row: Option<db::CaseRow> = sqlx::query_as("SELECT * FROM cases WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;

    let Some(row) = row else {
        return Err(ApiError::new("not_found", "case not found").into_response(StatusCode::NOT_FOUND));
    };

    ensure_case_visible(&authed, &row)?;

    let mut case_api = row.to_api();
    fill_stats(&state, std::slice::from_mut(&mut case_api)).await;
    Ok(Json(case_api))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCaseRequest {
    pub title: Option<String>,
    pub status: Option<crate::models::CaseStatus>,
    pub classification: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub async fn update(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCaseRequest>,
) -> Result<Json<Case>, Response> {
    let row: Option<db::CaseRow> = sqlx::query_as("SELECT * FROM cases WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;
    let Some(row) = row else {
        return Err(ApiError::new("not_found", "case not found").into_response(StatusCode::NOT_FOUND));
    };

    if authed.role != Role::Admin {
        authed.require(&[Role::Investigator])?;
        let owner_or_assignee =
            row.created_by == authed.id || row.assignee_ids().iter().any(|a| a == &authed.id);
        if !owner_or_assignee {
            return Err(ApiError::new("forbidden", "not your case")
                .into_response(StatusCode::FORBIDDEN));
        }
    }

    if let Some(title) = &req.title {
        sqlx::query("UPDATE cases SET title = ?2 WHERE id = ?1")
            .bind(id.to_string())
            .bind(title)
            .execute(&state.pool)
            .await
            .map_err(internal)?;
    }
    if let Some(status) = &req.status {
        let s = serde_json::to_string(status).unwrap_or_default();
        sqlx::query("UPDATE cases SET status = ?2 WHERE id = ?1")
            .bind(id.to_string())
            .bind(s.trim_matches('"'))
            .execute(&state.pool)
            .await
            .map_err(internal)?;
    }
    if let Some(classification) = &req.classification {
        sqlx::query("UPDATE cases SET classification = ?2 WHERE id = ?1")
            .bind(id.to_string())
            .bind(classification)
            .execute(&state.pool)
            .await
            .map_err(internal)?;
    }
    if let Some(tags) = &req.tags {
        sqlx::query("UPDATE cases SET tags = ?2 WHERE id = ?1")
            .bind(id.to_string())
            .bind(serde_json::to_string(tags).unwrap_or_else(|_| "[]".into()))
            .execute(&state.pool)
            .await
            .map_err(internal)?;
    }

    db::audit(
        &state.pool,
        &authed.id,
        Some(&id.to_string()),
        "case.updated",
        serde_json::json!({
            "title": req.title, "status": req.status,
            "classification": req.classification, "tags_changed": req.tags.is_some()
        }),
    )
    .await;

    let updated: db::CaseRow = sqlx::query_as("SELECT * FROM cases WHERE id = ?1")
        .bind(id.to_string())
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;
    Ok(Json(updated.to_api()))
}

pub async fn audit(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AuditEntry>>, Response> {
    authed.require(&[Role::Admin, Role::Supervisor])?;
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, user_id, action, detail, at FROM audit_log WHERE case_id = ?1 ORDER BY at DESC LIMIT 200",
    )
    .bind(id.to_string())
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;

    let entries = rows
        .into_iter()
        .map(|(eid, uid, action, detail, at)| AuditEntry {
            id: Uuid::parse_str(&eid).unwrap_or_default(),
            user_id: Uuid::parse_str(&uid).unwrap_or_default(),
            case_id: Some(id),
            action,
            detail: serde_json::from_str(&detail).unwrap_or_else(|_| serde_json::json!({})),
            at: chrono::DateTime::parse_from_rfc3339(&at)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
        .collect();

    Ok(Json(entries))
}

fn ensure_case_visible(authed: &Authed, row: &db::CaseRow) -> Result<(), Response> {
    match authed.role {
        Role::Admin | Role::Supervisor => Ok(()),
        _ => {
            let visible =
                row.created_by == authed.id || row.assignee_ids().iter().any(|a| a == &authed.id);
            if visible {
                Ok(())
            } else {
                Err(ApiError::new("not_found", "case not found").into_response(StatusCode::NOT_FOUND))
            }
        }
    }
}

async fn fill_stats(state: &AppState, cases: &mut [Case]) {
    if cases.is_empty() {
        return;
    }

    // Batch query: stats for all cases in one round-trip
    let case_ids: Vec<String> = cases.iter().map(|c| c.id.to_string()).collect();
    let placeholders: Vec<String> = case_ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let ph = placeholders.join(",");

    // Event counts by source_type per case
    let src_query = format!(
        "SELECT case_id, source_type, COUNT(*) as cnt FROM events WHERE case_id IN ({ph}) GROUP BY case_id, source_type"
    );
    let mut src_qb = sqlx::QueryBuilder::new(&src_query);
    for id in &case_ids {
        src_qb.push_bind(id.clone());
    }
    let src_rows: Vec<(String, String, i64)> = src_qb
        .build_query_as()
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    // Alert counts by severity per case
    let sev_query = format!(
        "SELECT case_id, severity, COUNT(*) as cnt FROM alerts WHERE case_id IN ({ph}) GROUP BY case_id, severity"
    );
    let mut sev_qb = sqlx::QueryBuilder::new(&sev_query);
    for id in &case_ids {
        sev_qb.push_bind(id.clone());
    }
    let sev_rows: Vec<(String, String, i64)> = sev_qb
        .build_query_as()
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    // Entity counts per case
    let ent_query = format!(
        "SELECT case_id, COUNT(*) as cnt FROM entities WHERE case_id IN ({ph}) GROUP BY case_id"
    );
    let mut ent_qb = sqlx::QueryBuilder::new(&ent_query);
    for id in &case_ids {
        ent_qb.push_bind(id.clone());
    }
    let ent_rows: Vec<(String, i64)> = ent_qb
        .build_query_as()
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    // Build lookup maps
    let mut src_map: std::collections::HashMap<String, std::collections::HashMap<String, i64>> = std::collections::HashMap::new();
    for (cid, src, cnt) in src_rows {
        src_map.entry(cid).or_default().insert(src, cnt);
    }
    let mut sev_map: std::collections::HashMap<String, std::collections::HashMap<String, i64>> = std::collections::HashMap::new();
    for (cid, sev, cnt) in sev_rows {
        sev_map.entry(cid).or_default().insert(sev, cnt);
    }
    let mut ent_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for (cid, cnt) in ent_rows {
        ent_map.insert(cid, cnt);
    }

    // Apply stats
    for c in cases.iter_mut() {
        let cid = c.id.to_string();
        if let Some(src) = src_map.remove(&cid) {
            let converted: Vec<(String, i64)> = src.into_iter().collect();
            c.stats.events_by_source = db::source_counts(converted);
        }
        if let Some(sev) = sev_map.remove(&cid) {
            let converted: Vec<(String, i64)> = sev.into_iter().collect();
            c.stats.alerts_by_severity = db::severity_counts(converted);
        }
        c.stats.entity_count = ent_map.remove(&cid).unwrap_or(0) as u64;
    }
}

pub async fn delete(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    authed.require(&[Role::Admin])?;

    let row: Option<db::CaseRow> = sqlx::query_as("SELECT * FROM cases WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;
    let Some(_case) = row else {
        return Err(ApiError::new("not_found", "case not found").into_response(StatusCode::NOT_FOUND));
    };

    let cid = id.to_string();

    // Cascade delete all related records
    sqlx::query("DELETE FROM events WHERE case_id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;
    sqlx::query("DELETE FROM entities WHERE case_id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;
    sqlx::query("DELETE FROM entity_edges WHERE case_id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;
    sqlx::query("DELETE FROM alerts WHERE case_id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;
    sqlx::query("DELETE FROM reports WHERE case_id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;
    sqlx::query("DELETE FROM ingest_jobs WHERE case_id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;
    sqlx::query("DELETE FROM feedback_queue WHERE case_id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;
    sqlx::query("DELETE FROM audit_log WHERE case_id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;

    // Delete the case itself
    sqlx::query("DELETE FROM cases WHERE id = ?1").bind(&cid).execute(&state.pool).await.map_err(internal)?;

    db::audit(
        &state.pool,
        &authed.id,
        None,
        "case.deleted",
        serde_json::json!({ "case_id": cid }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error").into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
