use chrono::{Duration, Utc};

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::models::{ModelInfo, TrainingQueueInfo, WebhookConfig};
use crate::state::AppState;

pub async fn get_webhooks() -> Json<WebhookConfig> {
    Json(WebhookConfig {
        discord_url: None,
        telegram_bot_token: None,
        telegram_chat_id: None,
    })
}

pub async fn update_webhooks() -> StatusCode {
    StatusCode::NO_CONTENT
}

pub async fn models() -> Json<Vec<ModelInfo>> {
    Json(vec![
        ModelInfo {
            version: "v1.0-base".into(),
            active: true,
            trained_at: Some(Utc::now() - Duration::days(3)),
            base_model: "mistral-7b-instruct-v0.3".into(),
        },
        ModelInfo {
            version: "v1.1-candidate".into(),
            active: false,
            trained_at: Some(Utc::now() - Duration::hours(20)),
            base_model: "mistral-7b-instruct-v0.3".into(),
        },
    ])
}

#[derive(Debug, serde::Deserialize)]
pub struct PromoteRequest {
    pub version: String,
}

pub async fn promote_model(
    State(state): State<AppState>,
    Json(req): Json<PromoteRequest>,
) -> StatusCode {
    state.publish("global", crate::models::WsEvent::ModelUpdated {
        payload: crate::models::ModelUpdated { version: req.version },
    });
    StatusCode::NO_CONTENT
}

pub async fn trigger_training(State(state): State<AppState>) -> StatusCode {
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
    StatusCode::ACCEPTED
}

pub async fn queue() -> Json<TrainingQueueInfo> {
    Json(TrainingQueueInfo {
        queued_events: 12,
        minimum_batch: 50,
        last_run: Some(Utc::now() - Duration::days(1)),
    })
}
