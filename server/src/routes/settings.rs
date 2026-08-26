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

fn internal<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!(err = %e, "internal error");
    ApiError::new("internal", "internal server error")
        .into_response(StatusCode::INTERNAL_SERVER_ERROR)
}
