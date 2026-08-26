use uuid::Uuid;

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::auth::Authed;
use crate::db;
use crate::models::{ApiError, GeneratedBy, Report, Role};
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct GenerateResponse {
    pub report_id: Uuid,
}

pub async fn generate(
    State(state): State<AppState>,
    authed: Authed,
    Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<GenerateResponse>), Response> {
    authed
        .require(&[Role::Admin, Role::Investigator, Role::Supervisor])?;

    let case_str = case_id.to_string();

    // Verify case exists
    let case_exists: Option<String> = sqlx::query_scalar("SELECT id FROM cases WHERE id = ?1")
        .bind(&case_str)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;
    if case_exists.is_none() {
        return Err(ApiError::new("not_found", "case not found")
            .into_response(StatusCode::NOT_FOUND));
    }

    // Compute report version
    let existing_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM reports WHERE case_id = ?1")
            .bind(&case_str)
            .fetch_one(&state.pool)
            .await
            .map_err(internal)?;
    let version = existing_count as u32 + 1;

    // Build template summary
    let summary_md = build_template_summary(&state, case_id).await?;

    let report_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO reports (id, case_id, version, generated_by, created_at, summary_md) VALUES (?1, ?2, ?3, 'template', ?4, ?5)",
    )
    .bind(report_id.to_string())
    .bind(&case_str)
    .bind(version)
    .bind(&now)
    .bind(&summary_md)
    .execute(&state.pool)
    .await
    .map_err(internal)?;

    db::audit(
        &state.pool,
        &authed.id,
        Some(&case_str),
        "report.generated",
        serde_json::json!({ "report_id": report_id.to_string(), "version": version }),
    )
    .await;

    Ok((StatusCode::CREATED, Json(GenerateResponse { report_id })))
}

async fn build_template_summary(
    state: &AppState,
    case_id: Uuid,
) -> Result<String, Response> {
    let cid = case_id.to_string();

    // Case info
    let case_row: Option<(String, String, String)> = sqlx::query_as(
        "SELECT title, status, classification FROM cases WHERE id = ?1",
    )
    .bind(&cid)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;
    let (title, status, classification) = case_row
        .ok_or_else(|| ApiError::new("not_found", "case not found").into_response(StatusCode::NOT_FOUND))?;

    // Event counts by source
    let src_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT source_type, COUNT(*) FROM events WHERE case_id = ?1 GROUP BY source_type",
    )
    .bind(&cid)
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;
    let total_events: i64 = src_rows.iter().map(|(_, c)| c).sum();

    // Entity count
    let entity_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM entities WHERE case_id = ?1")
            .bind(&cid)
            .fetch_one(&state.pool)
            .await
            .map_err(internal)?;

    // Alert counts by severity
    let sev_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT severity, COUNT(*) FROM alerts WHERE case_id = ?1 GROUP BY severity",
    )
    .bind(&cid)
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;
    let total_alerts: i64 = sev_rows.iter().map(|(_, c)| c).sum();

    // Top patterns
    let pattern_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT pattern, COUNT(*) FROM alerts WHERE case_id = ?1 GROUP BY pattern ORDER BY COUNT(*) DESC LIMIT 5",
    )
    .bind(&cid)
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;

    // Most-connected entities (top 5 by edge count)
    let top_entities: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT e.identifier, e.type, COUNT(ee.id) as cnt \
         FROM entities e \
         JOIN entity_edges ee ON (ee.source_entity_id = e.id OR ee.target_entity_id = e.id) \
         WHERE e.case_id = ?1 \
         GROUP BY e.id ORDER BY cnt DESC LIMIT 5",
    )
    .bind(&cid)
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;

    let generated_at = chrono::Utc::now().format("%d %B %Y, %H:%M UTC");

    let mut md = format!(
        "# Intelligence Report\n\n\
         **Case:** {title}\n\
         **Classification:** {classification}\n\
         **Status:** {status}\n\
         **Generated:** {generated_at}\n\
         **Report Type:** Template (auto-generated)\n\n\
         ---\n\n\
         ## Executive Summary\n\n\
         This case contains **{total_events} events** across {src_count} data sources, \
         **{entity_count} resolved entities**, and **{total_alerts} alerts**.\n\n",
        title = title,
        classification = classification,
        status = status,
        generated_at = generated_at,
        total_events = total_events,
        src_count = src_rows.len(),
        entity_count = entity_count,
        total_alerts = total_alerts,
    );

    // Source breakdown
    md += "## Data Sources\n\n";
    for (src, count) in &src_rows {
        md += &format!("- **{}:** {} events\n", src.to_uppercase(), count);
    }
    md += &format!("- **Total:** {} events\n\n", total_events);

    // Alert breakdown
    md += "## Alert Summary\n\n";
    for (sev, count) in &sev_rows {
        md += &format!("- **{}:** {} alerts\n", sev, count);
    }
    md += &format!("- **Total:** {} alerts\n\n", total_alerts);

    // Top patterns
    if !pattern_rows.is_empty() {
        md += "## Detected Patterns\n\n";
        for (pattern, count) in &pattern_rows {
            md += &format!("- **{}** (×{})\n", pattern, count);
        }
        md += "\n";
    }

    // Most connected entities
    if !top_entities.is_empty() {
        md += "## Key Entities (by connectivity)\n\n";
        for (identifier, etype, connections) in &top_entities {
            md += &format!(
                "- **{}** ({}): {} connections\n",
                identifier,
                etype,
                connections
            );
        }
        md += "\n";
    }

    md += "---\n\n\
           *This is a template-generated report. For LLM-enhanced analysis, \
           integrate a language model backend.*\n";

    Ok(md)
}

pub async fn list(
    State(state): State<AppState>,
    _authed: Authed,
    Path(case_id): Path<Uuid>,
) -> Result<Json<Vec<Report>>, Response> {
    let rows: Vec<(String, String, i64, String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, case_id, version, generated_by, approved_by, created_at, summary_md FROM reports WHERE case_id = ?1 ORDER BY version DESC",
    )
    .bind(case_id.to_string())
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, cid, ver, gen_by, approved, at, md)| Report {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                case_id: Uuid::parse_str(&cid).unwrap_or_default(),
                version: ver as u32,
                generated_by: if gen_by == "llm" {
                    GeneratedBy::Llm
                } else {
                    GeneratedBy::Template
                },
                approved_by: approved.and_then(|a| Uuid::parse_str(&a).ok()),
                created_at: chrono::DateTime::parse_from_rfc3339(&at)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                summary_md: md,
            })
            .collect(),
    ))
}

pub async fn detail(
    State(state): State<AppState>,
    _authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<Report>>, Response> {
    let row: Option<(String, String, i64, String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, case_id, version, generated_by, approved_by, created_at, summary_md FROM reports WHERE id = ?1",
    )
    .bind(id.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;

    Ok(Json(row.map(|(id, cid, ver, gen_by, approved, at, md)| {
        Report {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            case_id: Uuid::parse_str(&cid).unwrap_or_default(),
            version: ver as u32,
            generated_by: if gen_by == "llm" {
                GeneratedBy::Llm
            } else {
                GeneratedBy::Template
            },
            approved_by: approved.and_then(|a| Uuid::parse_str(&a).ok()),
            created_at: chrono::DateTime::parse_from_rfc3339(&at)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            summary_md: md,
        }
    })))
}

pub async fn export_pdf(
    State(state): State<AppState>,
    _authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    let row: Option<String> = sqlx::query_scalar(
        "SELECT summary_md FROM reports WHERE id = ?1",
    )
    .bind(id.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;

    let Some(md) = row else {
        return Err(
            ApiError::new("not_found", "report not found").into_response(StatusCode::NOT_FOUND),
        );
    };

    // Export as markdown file (PDF generation requires external tooling)
    let body = md.into_bytes();
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/markdown; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"netra-report.md\"",
            ),
        ],
        body,
    )
        .into_response())
}

pub async fn approve(
    State(state): State<AppState>,
    authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<Json<Report>, Response> {
    authed.require(&[Role::Admin, Role::Supervisor])?;

    let _now = chrono::Utc::now().to_rfc3339();
    let res = sqlx::query("UPDATE reports SET approved_by = ?2 WHERE id = ?1")
        .bind(id.to_string())
        .bind(&authed.id)
        .execute(&state.pool)
        .await
        .map_err(internal)?;

    if res.rows_affected() == 0 {
        return Err(
            ApiError::new("not_found", "report not found").into_response(StatusCode::NOT_FOUND),
        );
    }

    db::audit(
        &state.pool,
        &authed.id,
        None,
        "report.approved",
        serde_json::json!({ "report_id": id.to_string() }),
    )
    .await;

    let row: (String, String, i64, String, Option<String>, String, String) = sqlx::query_as(
        "SELECT id, case_id, version, generated_by, approved_by, created_at, summary_md FROM reports WHERE id = ?1",
    )
    .bind(id.to_string())
    .fetch_one(&state.pool)
    .await
    .map_err(internal)?;

    Ok(Json(report_from_row(row)))
}

fn report_from_row(
    (id, cid, ver, gen_by, approved, at, md): (String, String, i64, String, Option<String>, String, String),
) -> Report {
    Report {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        case_id: Uuid::parse_str(&cid).unwrap_or_default(),
        version: ver as u32,
        generated_by: if gen_by == "llm" {
            GeneratedBy::Llm
        } else {
            GeneratedBy::Template
        },
        approved_by: approved.and_then(|a| Uuid::parse_str(&a).ok()),
        created_at: chrono::DateTime::parse_from_rfc3339(&at)
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        summary_md: md,
    }
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
