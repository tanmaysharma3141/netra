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
    let report_id = Uuid::new_v4();
    let sha256 = format!("NETRA-RPT-{}", &report_id.to_string()[..8]);

    // CrPC 65B compliant header
    let mut md = format!(
        "# INTELLIGENCE REPORT — CrPC Section 65B Compliant\n\n\
         ---\n\n\
         **Court / Authority:** _________________________________\n\
         **Case Reference No.:** {title}\n\
         **Classification:** {classification}\n\
         **Status:** {status}\n\
         **Report ID:** {sha256}\n\
         **Generated On:** {generated_at}\n\
         **Prepared By:** NETRA Automated Intelligence Platform v0.1\n\
         **Report Type:** Template (auto-generated, system-verified)\n\n\
         ---\n\n\
         ## 1. Executive Summary\n\n\
         This intelligence report has been generated by the NETRA forensic intelligence \
         platform in compliance with Section 65B of the Indian Evidence Act, 1872 \
         (as amended by the Information Technology Act, 2000).\n\n\
         The investigation case contains **{total_events} events** across {src_count} data sources, \
         **{entity_count} resolved entities**, and **{total_alerts} automated alerts** \
         indicating suspicious patterns.\n\n",
        title = title,
        classification = classification,
        status = status,
        generated_at = generated_at,
        sha256 = sha256,
        total_events = total_events,
        src_count = src_rows.len(),
        entity_count = entity_count,
        total_alerts = total_alerts,
    );

    // Source breakdown
    md += "## 2. Data Sources Analyzed\n\n";
    for (src, count) in &src_rows {
        md += &format!("- **{}:** {} records\n", src.to_uppercase(), count);
    }
    md += &format!("- **Total records analyzed:** {}\n\n", total_events);

    // Alert breakdown
    md += "## 3. Automated Alert Summary\n\n";
    md += "The following anomalies were detected by NETRA's rule engine:\n\n";
    for (sev, count) in &sev_rows {
        md += &format!("- **{} severity:** {} alerts\n", sev, count);
    }
    md += &format!("- **Total alerts:** {}\n\n", total_alerts);

    // Top patterns
    if !pattern_rows.is_empty() {
        md += "## 4. Detected Patterns\n\n";
        md += "The following suspicious patterns were identified in the data:\n\n";
        for (pattern, count) in &pattern_rows {
            md += &format!("- **{}** (×{} occurrences)\n", pattern, count);
        }
        md += "\n";
    }

    // Most connected entities
    if !top_entities.is_empty() {
        md += "## 5. Key Entities (by connectivity)\n\n";
        md += "Entities with the highest number of connections in the communication/financial network:\n\n";
        for (identifier, etype, connections) in &top_entities {
            md += &format!(
                "- **{}** ({}, {} connections)\n",
                identifier,
                etype,
                connections
            );
        }
        md += "\n";
    }

    // CrPC 65B compliance footer
    md += "## 6. Declaration under Section 65B, Indian Evidence Act\n\n";
    md += &format!(
        "I, the undersigned, certify that this report was generated by the NETRA \
         Automated Intelligence Platform (v0.1), a computer system within the meaning \
         of Section 3 of the Information Technology Act, 2000.\n\n\
         The data contained herein was extracted from electronic records maintained \
         in the ordinary course of investigation. The integrity of the source data \
         was verified using SHA-256 cryptographic hashing at the time of ingestion.\n\n\
         This report is generated on: {generated_at}\n\
         Report ID: {sha256}\n\n\
         ---\n\n\
         **Signature:** _________________________________\n\
         **Name & Designation:** _________________________________\n\
         **Date:** _________________________________\n\n"
    );

    md += "---\n\n\
           *This report was auto-generated by NETRA v0.1. For questions regarding \
           methodology or data sources, contact the investigating authority.*\n";

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
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT summary_md, title FROM reports r JOIN cases c ON r.case_id = c.id WHERE r.id = ?1",
    )
    .bind(id.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;

    let Some((md, title)) = row else {
        return Err(
            ApiError::new("not_found", "report not found").into_response(StatusCode::NOT_FOUND),
        );
    };

    let pdf_bytes = crate::pdf::generate_report_pdf(&title, &md);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"netra-report.pdf\"",
            ),
        ],
        pdf_bytes,
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
