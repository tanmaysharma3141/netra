use std::path::Path as StdPath;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::Authed;
use crate::db;
use crate::models::{ApiError, IngestJob, IngestJobStatus, Role, WsEvent};
use crate::state::AppState;

const UPLOAD_DIR: &str = "data/uploads";
const BATCH_ROWS: usize = 60;

#[derive(serde::Serialize)]
pub struct UploadAccepted {
    job_id: Uuid,
}

pub async fn upload(
    State(state): State<AppState>,
    authed: Authed,
    Path(case_id): Path<Uuid>,
    mut multipart: axum::extract::multipart::Multipart,
) -> Result<(StatusCode, Json<UploadAccepted>), Response> {
    authed.require(&[Role::Admin, Role::Investigator])?;

    std::fs::create_dir_all(UPLOAD_DIR).map_err(internal)?;

    let mut first_job: Option<Uuid> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::new("bad_request", format!("multipart error: {e}")).into_response(StatusCode::BAD_REQUEST))?
    {
        let Some(file_name) = field.file_name().map(String::from) else {
            continue;
        };

        // Validate file extension
        let ext = std::path::Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !matches!(ext.as_str(), "csv" | "tsv" | "txt" | "zip" | "xlsx" | "xls" | "pdf" | "docx") {
            return Err(ApiError::new(
                "bad_request",
                format!("unsupported file type '.{ext}'; accepted: .csv, .tsv, .txt, .zip, .xlsx, .xls, .pdf, .docx"),
            )
            .into_response(StatusCode::BAD_REQUEST));
        }

        // Sanitize filename: strip path separators, limit length
        let safe_name = file_name
            .chars()
            .filter(|c| !matches!(c, '/' | '\\' | '\0'))
            .take(255)
            .collect::<String>();

        let job_id = Uuid::new_v4();
        let save_path = StdPath::new(UPLOAD_DIR).join(format!("{job_id}.bin"));

        let mut hasher = Sha256::new();
        let file = std::fs::File::create(&save_path).map_err(internal)?;
        let mut writer = std::io::BufWriter::new(file);
        use std::io::Write;
        let mut stream = field;
        while let Some(chunk) = stream.chunk().await.map_err(internal)? {
            writer.write_all(&chunk).map_err(internal)?;
            hasher.update(&chunk);
        }
        writer.flush().map_err(internal)?;

        let sha256 = hex::encode(hasher.finalize());
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("INSERT INTO ingest_jobs (id, case_id, status, file_name, sha256, records_parsed, total_est, errors, started_at) VALUES (?1, ?2, 'queued', ?3, ?4, 0, 0, '[]', ?5)")
            .bind(job_id.to_string())
            .bind(case_id.to_string())
            .bind(&safe_name)
            .bind(&sha256)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(internal)?;

        db::audit(
            &state.pool,
            &authed.id,
            Some(&case_id.to_string()),
            "ingest.uploaded",
            serde_json::json!({ "file": safe_name, "sha256": sha256 }),
        )
        .await;

        let st = state.clone();
        let spawn_name = safe_name.clone();
        tokio::spawn(async move {
            run_job(st, case_id, job_id, save_path, spawn_name, sha256).await;
        });

        if first_job.is_none() {
            first_job = Some(job_id);
        }
    }

    match first_job {
        Some(job_id) => Ok((StatusCode::ACCEPTED, Json(UploadAccepted { job_id }))),
        None => Err(ApiError::new("bad_request", "no file parts found")
            .into_response(StatusCode::BAD_REQUEST)),
    }
}

async fn run_job(
    state: AppState,
    case_id: Uuid,
    job_id: Uuid,
    path: std::path::PathBuf,
    file_name: String,
    sha256: String,
) {
    let topic = format!("case:{case_id}");
    set_status(&state.pool, job_id, "running").await;

    let progress_state = state.clone();
    let progress_topic = topic.clone();
    let parse_path = path.clone();
    let parse_name = file_name.clone();
    let parse_result = tokio::task::spawn_blocking(move || {
        crate::ingest::run_parse(&parse_path, case_id, &parse_name, move |parsed| {
            progress_state.publish(
                progress_topic.clone(),
                WsEvent::IngestProgress {
                    payload: crate::models::IngestProgress { job_id, parsed, total_est: parsed },
                },
            );
        })
    })
    .await;

    let result = match parse_result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(err = %e, "ingest task panicked");
            fail_job(&state.pool, job_id, vec![format!("internal: {e}")]).await;
            return;
        }
    };

    match result {
        Ok(res) => {
            if res.parsed == 0 && !res.errors.is_empty() {
                fail_job(&state.pool, job_id, res.errors).await;
                return;
            }

            let inserted = insert_events(&state.pool, case_id, &res.events).await;
            if let Err(e) = inserted {
                tracing::error!(err = %e, "event insert failed");
                fail_job(&state.pool, job_id, vec![format!("db insert failed: {e}")]).await;
                return;
            }

            let _ = sqlx::query(
                "UPDATE ingest_jobs SET status='done', records_parsed=?2, errors=?3, finished_at=?4 WHERE id=?1",
            )
            .bind(job_id.to_string())
            .bind(res.parsed as i64)
            .bind(serde_json::to_string(&res.errors).unwrap_or_else(|_| "[]".into()))
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&state.pool)
            .await;

            db::audit(
                &state.pool,
                SYSTEM_USER,
                Some(&case_id.to_string()),
                "ingest.completed",
                serde_json::json!({
                    "file": file_name, "sha256": sha256,
                    "records": res.parsed, "domain": res.domain,
                    "operator": res.operator
                }),
            )
            .await;

            tracing::info!(file = %file_name, parsed = res.parsed, domain = res.domain, "ingest done");

            // Pipeline: resolve → analyze → push alerts
            // Uses pipeline_lock to serialize across all cases
            let analyze_state = state.clone();
            let resolve_pool = state.pool.clone();
            tokio::spawn(async move {
                let _guard = analyze_state.pipeline_lock.lock().await;

                // Check if another job already resolved this case recently (skip redundant)
                // Resolve is idempotent (full rebuild) — pipeline_lock serializes across cases
                match crate::resolve::resolve_case(&resolve_pool, case_id).await {
                    Ok(s) => {
                        tracing::info!(
                            entities = s.entities, edges = s.edges,
                            device_links = s.device_links, comm_links = s.communication_links,
                            "auto-resolution complete"
                        );
                        match crate::anomaly::analyze_case(&resolve_pool, case_id).await {
                            Ok(stats) => {
                                tracing::info!(alerts = stats.alerts_raised, "auto-analysis complete");
                                let open: Vec<(String,)> = sqlx::query_as(
                                    "SELECT id FROM alerts WHERE case_id = ?1 AND status = 'open' ORDER BY created_at DESC LIMIT 10",
                                )
                                .bind(case_id.to_string())
                                .fetch_all(&analyze_state.pool)
                                .await
                                .unwrap_or_default();
                                let mut webhook_ids = Vec::new();
                                for (aid,) in open {
                                    if let Ok(uuid) = Uuid::parse_str(&aid) {
                                        if let Some(alert) = crate::routes::alerts::fetch_alert(&analyze_state.pool, uuid).await {
                                            analyze_state.publish(
                                                format!("case:{case_id}"),
                                                crate::models::WsEvent::AlertCreated { payload: alert },
                                            );
                                            webhook_ids.push(uuid);
                                        }
                                    }
                                }
                                // Fire webhook notifications
                                if !webhook_ids.is_empty() {
                                    crate::webhook::notify_new_alerts(analyze_state.pool.clone(), case_id, webhook_ids);
                                }
                            }
                            Err(e) => tracing::error!(err = %e, "auto-analysis failed"),
                        }
                    }
                    Err(e) => tracing::error!(err = %e, "auto-resolution failed"),
                }
            });

            let _ = std::fs::remove_file(&path);
        }
        Err(e) => fail_job(&state.pool, job_id, vec![e]).await,
    }
}

const SYSTEM_USER: &str = "22222222-2222-2222-2222-222222222222";

async fn insert_events(
    pool: &sqlx::SqlitePool,
    case_id: Uuid,
    events: &[crate::models::Event],
) -> Result<u64, sqlx::Error> {
    use sqlx::QueryBuilder;
    let case_str = case_id.to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let mut total = 0u64;

    for batch in events.chunks(BATCH_ROWS) {
        let mut qb = QueryBuilder::new(
            "INSERT OR IGNORE INTO events (\"id\", \"case_id\", \"ts\", \"source_type\", \"entity_id\", \"entity_type\", \"event_type\", \"value\", \"lat\", \"lng\", \"raw\", \"ingested_at\", \"notes\") ",
        );
        qb.push_values(batch, |mut b, ev| {
            b.push_bind(ev.id.to_string())
                .push_bind(case_str.clone())
                .push_bind(ev.timestamp.to_rfc3339())
                .push_bind(ev.source_type.db_str())
                .push_bind(ev.entity_id.clone())
                .push_bind(ev.entity_type.db_str())
                .push_bind(ev.event_type.db_str())
                .push_bind(ev.value)
                .push_bind(ev.location.map(|l| l.lat))
                .push_bind(ev.location.map(|l| l.lng))
                .push_bind(ev.raw.to_string())
                .push_bind(now.clone())
                .push_bind("[]");
        });
        total += qb.build().execute(pool).await?.rows_affected();
    }
    Ok(total)
}

async fn set_status(pool: &sqlx::SqlitePool, job_id: Uuid, status: &str) {
    let _ = sqlx::query("UPDATE ingest_jobs SET status=?2 WHERE id=?1")
        .bind(job_id.to_string())
        .bind(status)
        .execute(pool)
        .await;
}

async fn fail_job(pool: &sqlx::SqlitePool, job_id: Uuid, errors: Vec<String>) {
    let _ = sqlx::query("UPDATE ingest_jobs SET status='failed', errors=?2, finished_at=?3 WHERE id=?1")
        .bind(job_id.to_string())
        .bind(serde_json::to_string(&errors).unwrap_or_else(|_| "[]".into()))
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(pool)
        .await;
    tracing::warn!(job = %job_id, first_error = ?errors.first(), "ingest failed");
}

pub async fn job(
    State(state): State<AppState>,
    _authed: Authed,
    Path(id): Path<Uuid>,
) -> Result<Json<IngestJob>, Response> {
    let row: Option<(String, String, String, i64, String)> = sqlx::query_as(
        "SELECT id, case_id, status, records_parsed, errors FROM ingest_jobs WHERE id = ?1",
    )
    .bind(id.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;

    let Some((jid, cid, status, parsed, errors)) = row else {
        return Err(ApiError::new("not_found", "job not found").into_response(StatusCode::NOT_FOUND));
    };

    Ok(Json(IngestJob {
        id: Uuid::parse_str(&jid).unwrap_or_default(),
        case_id: Uuid::parse_str(&cid).unwrap_or_default(),
        status: status.parse().unwrap_or(IngestJobStatus::Queued),
        records_parsed: parsed as u64,
        errors: serde_json::from_str(&errors).unwrap_or_default(),
    }))
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error").into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
