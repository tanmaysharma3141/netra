use std::io::Write;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;
use zip::write::SimpleFileOptions;

use crate::auth::Authed;
use crate::models::{ApiError, Role};
use crate::state::AppState;

pub async fn export_case(
    State(state): State<AppState>,
    authed: Authed,
    Path(case_id): Path<Uuid>,
) -> Result<Response, Response> {
    authed.require(&[Role::Admin, Role::Supervisor, Role::Investigator])?;

    let cid = case_id.to_string();

    // Verify case exists
    let case: Option<(String, String)> = sqlx::query_as("SELECT id, title FROM cases WHERE id = ?1")
        .bind(&cid)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;
    let (case_id_str, case_title) = case.ok_or_else(|| {
        ApiError::new("not_found", "case not found").into_response(StatusCode::NOT_FOUND)
    })?;

    // Build zip in memory
    let zip_buf: Vec<u8> = Vec::new();
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(zip_buf));
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. Events CSV
    let events: Vec<(String, String, String, String, String, String, Option<f64>, String)> =
        sqlx::query_as("SELECT id, ts, source_type, entity_id, entity_type, event_type, value, raw FROM events WHERE case_id = ?1 ORDER BY ts")
            .bind(&cid)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;

    let mut events_csv = String::from("id,ts,source_type,entity_id,entity_type,event_type,value\n");
    for ev in &events {
        events_csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            ev.0, ev.1, ev.2, ev.3, ev.4, ev.5,
            ev.6.map(|v| v.to_string()).unwrap_or_default()
        ));
    }
    zip.start_file("events.csv", options).map_err(zip_err)?;
    zip.write_all(events_csv.as_bytes()).map_err(zip_err)?;

    // 2. Entities CSV
    let entities: Vec<(String, String, String, Option<String>, Option<String>, String)> =
        sqlx::query_as("SELECT id, type, identifier, display_name, link_tier, tags FROM entities WHERE case_id = ?1")
            .bind(&cid)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;

    let mut entities_csv = String::from("id,type,identifier,display_name,link_tier,tags\n");
    for ent in &entities {
        entities_csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            ent.0, ent.1, ent.2,
            ent.3.as_deref().unwrap_or(""),
            ent.4.as_deref().unwrap_or(""),
            ent.5
        ));
    }
    zip.start_file("entities.csv", options).map_err(zip_err)?;
    zip.write_all(entities_csv.as_bytes()).map_err(zip_err)?;

    // 3. Edges CSV
    let edges: Vec<(String, String, String, String, f64, i64)> =
        sqlx::query_as("SELECT source_entity_id, target_entity_id, link_type, tier, confidence, evidence_count FROM entity_edges WHERE case_id = ?1")
            .bind(&cid)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;

    let mut edges_csv = String::from("source,target,link_type,tier,confidence,evidence_count\n");
    for edge in &edges {
        edges_csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            edge.0, edge.1, edge.2, edge.3, edge.4, edge.5
        ));
    }
    zip.start_file("edges.csv", options).map_err(zip_err)?;
    zip.write_all(edges_csv.as_bytes()).map_err(zip_err)?;

    // 4. Alerts JSON
    let alerts: Vec<(String, String, String, i64, String, String, String, String, String)> =
        sqlx::query_as("SELECT id, pattern, severity, score, status, entity_ids, evidence_event_ids, summary, created_at FROM alerts WHERE case_id = ?1")
            .bind(&cid)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;

    let alerts_json = serde_json::to_string_pretty(&alerts.iter().map(|a| {
        serde_json::json!({
            "id": a.0, "pattern": a.1, "severity": a.2, "score": a.3,
            "status": a.4, "entity_ids": a.5, "evidence_event_ids": a.6,
            "summary": a.7, "created_at": a.8
        })
    }).collect::<Vec<_>>()).unwrap_or_default();
    zip.start_file("alerts.json", options).map_err(zip_err)?;
    zip.write_all(alerts_json.as_bytes()).map_err(zip_err)?;

    // 5. Audit log JSON
    let audit: Vec<(String, String, String, String, String, String)> =
        sqlx::query_as("SELECT id, user_id, case_id, action, detail, at FROM audit_log WHERE case_id = ?1 ORDER BY at")
            .bind(&cid)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;

    let audit_json = serde_json::to_string_pretty(&audit.iter().map(|a| {
        serde_json::json!({
            "id": a.0, "user_id": a.1, "case_id": a.2, "action": a.3, "detail": a.4, "at": a.5
        })
    }).collect::<Vec<_>>()).unwrap_or_default();
    zip.start_file("audit_log.json", options).map_err(zip_err)?;
    zip.write_all(audit_json.as_bytes()).map_err(zip_err)?;

    // Finalize zip
    let cursor = zip.finish().map_err(zip_err)?;
    let zip_bytes = cursor.into_inner();

    // Log export action
    crate::db::audit(
        &state.pool,
        &authed.id,
        Some(&case_id_str),
        "case.exported",
        serde_json::json!({ "events": events.len(), "entities": entities.len(), "edges": edges.len() }),
    )
    .await;

    // Return as download
    let filename = format!("netra-case-{}.zip", case_title.replace(' ', "_"));
    let headers = [
        ("Content-Type", "application/zip"),
        ("Content-Disposition", &format!("attachment; filename=\"{filename}\"")),
    ];

    Ok((headers, zip_bytes).into_response())
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}

fn zip_err(e: impl std::fmt::Display) -> Response {
    tracing::error!(err = %e, "zip creation failed");
    ApiError::new("internal", "failed to create export archive")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
