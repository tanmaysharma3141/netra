use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;

use crate::auth::Authed;
use crate::db;
use crate::models::{ApiError, ModelInfo, Role, TrainingQueueInfo, WebhookConfig};
use crate::state::AppState;

// --- Webhooks ---

pub async fn get_webhooks(
    State(state): State<AppState>,
    _authed: Authed,
) -> Result<Json<WebhookConfig>, Response> {
    let row: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT discord_url, telegram_bot_token, telegram_chat_id FROM webhook_configs LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;

    Ok(Json(match row {
        Some((d, t, c)) => WebhookConfig {
            discord_url: d,
            telegram_bot_token: t,
            telegram_chat_id: c,
        },
        None => WebhookConfig {
            discord_url: None,
            telegram_bot_token: None,
            telegram_chat_id: None,
        },
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct WebhookUpdate {
    pub discord_url: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
}

pub async fn update_webhooks(
    State(state): State<AppState>,
    authed: Authed,
    Json(req): Json<WebhookUpdate>,
) -> Result<StatusCode, Response> {
    authed.require(&[Role::Admin])?;

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE webhook_configs SET discord_url = ?2, telegram_bot_token = ?3, telegram_chat_id = ?4, updated_at = ?5 WHERE id = '00000000-0000-0000-0000-000000000001'",
    )
    .bind(&now)
    .bind(&req.discord_url)
    .bind(&req.telegram_bot_token)
    .bind(&req.telegram_chat_id)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(internal)?;

    db::audit(
        &state.pool,
        &authed.id,
        None,
        "settings.webhooks_updated",
        serde_json::json!({
            "discord_url_set": req.discord_url.is_some(),
            "telegram_set": req.telegram_bot_token.is_some(),
        }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

// --- Models ---

pub async fn models(
    State(state): State<AppState>,
    _authed: Authed,
) -> Result<Json<Vec<ModelInfo>>, Response> {
    let rows: Vec<(String, String, i64, Option<String>, String)> = sqlx::query_as(
        "SELECT id, version, active, trained_at, base_model FROM models ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;

    Ok(Json(
        rows.into_iter()
            .map(|(_id, version, active, trained_at, base_model)| ModelInfo {
                version,
                active: active != 0,
                trained_at: trained_at.and_then(|t| {
                    chrono::DateTime::parse_from_rfc3339(&t)
                        .map(|d| d.with_timezone(&chrono::Utc))
                        .ok()
                }),
                base_model,
            })
            .collect(),
    ))
}

pub async fn promote_model(
    State(state): State<AppState>,
    authed: Authed,
    Json(req): Json<crate::routes::settings::PromoteRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    authed.require(&[Role::Admin])?;

    // Check version exists
    let exists: Option<String> = sqlx::query_scalar("SELECT id FROM models WHERE version = ?1")
        .bind(&req.version)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?;
    if exists.is_none() {
        return Err(
            ApiError::new("not_found", "model version not found").into_response(StatusCode::NOT_FOUND),
        );
    }

    // Deactivate all, activate target
    sqlx::query("UPDATE models SET active = 0")
        .execute(&state.pool)
        .await
        .map_err(internal)?;
    sqlx::query("UPDATE models SET active = 1 WHERE version = ?1")
        .bind(&req.version)
        .execute(&state.pool)
        .await
        .map_err(internal)?;

    state.publish(
        "global",
        crate::models::WsEvent::ModelUpdated {
            payload: crate::models::ModelUpdated {
                version: req.version.clone(),
            },
        },
    );

    db::audit(
        &state.pool,
        &authed.id,
        None,
        "model.promoted",
        serde_json::json!({ "version": req.version }),
    )
    .await;

    Ok(Json(serde_json::json!({ "promoted": req.version })))
}

// --- Training ---

pub async fn trigger_training(
    State(state): State<AppState>,
    authed: Authed,
) -> Result<StatusCode, Response> {
    authed.require(&[Role::Admin, Role::Analyst])?;

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE training_queue SET last_run = ?1")
        .bind(&now)
        .execute(&state.pool)
        .await
        .map_err(internal)?;

    db::audit(
        &state.pool,
        &authed.id,
        None,
        "training.triggered",
        serde_json::json!({}),
    )
    .await;

    // Spawn simulated training progress (placeholder until real LLM integration)
    let tx = state.tx.clone();
    tokio::spawn(async move {
        for epoch in 1..=3u32 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = tx.send(crate::models::WsEnvelope {
                topic: "global".into(),
                event: crate::models::WsEvent::TrainingProgress {
                    payload: crate::models::TrainingProgress {
                        epoch,
                        loss: 0.6 - f64::from(epoch) * 0.1,
                        stage: "training".into(),
                    },
                },
            });
        }
    });

    Ok(StatusCode::ACCEPTED)
}

pub async fn queue(
    State(state): State<AppState>,
    _authed: Authed,
) -> Result<Json<TrainingQueueInfo>, Response> {
    let row: Option<(i64, i64, Option<String>)> =
        sqlx::query_as("SELECT queued_events, minimum_batch, last_run FROM training_queue LIMIT 1")
            .fetch_optional(&state.pool)
            .await
            .map_err(internal)?;

    Ok(Json(match row {
        Some((queued, min_batch, last_run)) => TrainingQueueInfo {
            queued_events: queued as u64,
            minimum_batch: min_batch as u64,
            last_run: last_run.and_then(|t| {
                chrono::DateTime::parse_from_rfc3339(&t)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .ok()
            }),
        },
        None => TrainingQueueInfo {
            queued_events: 0,
            minimum_batch: 50,
            last_run: None,
        },
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct PromoteRequest {
    pub version: String,
}

// --- Alert Thresholds ---

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AlertThresholds {
    pub imei_min_subscribers: usize,
    pub imei_min_evidence: i64,
    pub hawala_window_hours: i64,
    pub hawala_min_txns: usize,
    pub hawala_min_total: f64,
    pub hawala_max_total: f64,
    pub rapid_window_minutes: i64,
    pub rapid_min_txns: usize,
    pub rapid_min_flow: f64,
    pub silence_min_parties: usize,
    pub bot_min_posts: usize,
    pub bot_max_interval_secs: i64,
    pub round_trip_window_hours: i64,
    pub tower_jump_max_minutes: i64,
    pub tower_jump_min_km: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            imei_min_subscribers: 3,
            imei_min_evidence: 40,
            hawala_window_hours: 48,
            hawala_min_txns: 4,
            hawala_min_total: 40000.0,
            hawala_max_total: 150000.0,
            rapid_window_minutes: 60,
            rapid_min_txns: 3,
            rapid_min_flow: 300000.0,
            silence_min_parties: 3,
            bot_min_posts: 10,
            bot_max_interval_secs: 300,
            round_trip_window_hours: 48,
            tower_jump_max_minutes: 30,
            tower_jump_min_km: 50.0,
        }
    }
}

pub async fn get_alert_thresholds(
    State(state): State<AppState>,
    _authed: Authed,
) -> Result<Json<AlertThresholds>, Response> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'alert_thresholds' LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;

    match row {
        Some((json_str,)) => {
            let thresholds: AlertThresholds = serde_json::from_str(&json_str)
                .unwrap_or_default();
            Ok(Json(thresholds))
        }
        None => Ok(Json(AlertThresholds::default())),
    }
}

pub async fn update_alert_thresholds(
    State(state): State<AppState>,
    authed: Authed,
    Json(thresholds): Json<AlertThresholds>,
) -> Result<StatusCode, Response> {
    authed.require(&[Role::Admin])?;

    let json_str = serde_json::to_string(&thresholds).map_err(internal)?;
    sqlx::query(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('alert_thresholds', ?1)",
    )
    .bind(&json_str)
    .execute(&state.pool)
    .await
    .map_err(internal)?;

    db::audit(
        &state.pool,
        &authed.id,
        None,
        "settings.alert_thresholds_updated",
        serde_json::json!({}),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

// --- Data Retention ---

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RetentionConfig {
    pub archive_after_days: u64,
    pub delete_after_days: u64,
    pub enabled: bool,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            archive_after_days: 365,
            delete_after_days: 730,
            enabled: false,
        }
    }
}

pub async fn get_retention(
    State(state): State<AppState>,
    _authed: Authed,
) -> Result<Json<RetentionConfig>, Response> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'retention' LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;

    match row {
        Some((json_str,)) => {
            let config: RetentionConfig = serde_json::from_str(&json_str)
                .unwrap_or_default();
            Ok(Json(config))
        }
        None => Ok(Json(RetentionConfig::default())),
    }
}

pub async fn update_retention(
    State(state): State<AppState>,
    authed: Authed,
    Json(config): Json<RetentionConfig>,
) -> Result<StatusCode, Response> {
    authed.require(&[Role::Admin])?;

    let json_str = serde_json::to_string(&config).map_err(internal)?;
    sqlx::query(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('retention', ?1)",
    )
    .bind(&json_str)
    .execute(&state.pool)
    .await
    .map_err(internal)?;

    db::audit(
        &state.pool,
        &authed.id,
        None,
        "settings.retention_updated",
        serde_json::json!({}),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
