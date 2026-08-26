use std::time::Duration;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{Alert, Severity};

/// Configuration loaded from DB
#[derive(Debug, Clone, Default)]
pub struct WebhookConfig {
    pub discord_url: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
}

/// Load webhook config from the database
pub async fn load_config(pool: &SqlitePool) -> WebhookConfig {
    let row: Option<(Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT discord_url, telegram_bot_token, telegram_chat_id FROM webhook_configs LIMIT 1")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    match row {
        Some((d, t, c)) => WebhookConfig {
            discord_url: d,
            telegram_bot_token: t,
            telegram_chat_id: c,
        },
        None => WebhookConfig::default(),
    }
}

/// Main entry point: notify about new alerts.
/// Spawned as a tokio task — non-blocking, with retry.
pub fn notify_new_alerts(pool: SqlitePool, case_id: Uuid, alert_ids: Vec<Uuid>) {
    tokio::spawn(async move {
        let config = load_config(&pool).await;
        let has_discord = config.discord_url.is_some();
        let has_telegram = config.telegram_bot_token.is_some() && config.telegram_chat_id.is_some();

        if !has_discord && !has_telegram {
            return; // No webhooks configured, skip silently
        }

        // Load alert details
        let alerts = match load_alerts(&pool, &alert_ids).await {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!(err = %e, "failed to load alerts for webhook notification");
                return;
            }
        };

        if alerts.is_empty() {
            return;
        }

        // Load case title
        let case_title = load_case_title(&pool, case_id).await.unwrap_or_default();

        // Send to Discord (batched into one message with multiple embeds)
        if has_discord {
            let url = config.discord_url.clone().unwrap();
            let embeds: Vec<serde_json::Value> = alerts.iter().map(|a| discord_embed(a, &case_title)).collect();

            // Discord allows max 10 embeds per message
            for chunk in embeds.chunks(10) {
                let payload = serde_json::json!({
                    "embeds": chunk
                });
                let url_clone = url.clone();
                let payload_clone = payload.clone();
                retry_send(move || {
                    let url = url_clone.clone();
                    let payload = payload_clone.clone();
                    async move {
                        let client = reqwest::Client::new();
                        client.post(&url)
                            .json(&payload)
                            .timeout(Duration::from_secs(10))
                            .send()
                            .await
                    }
                }, 3).await;
            }
        }

        // Send to Telegram (one message per alert)
        if has_telegram {
            let token = config.telegram_bot_token.clone().unwrap();
            let chat_id = config.telegram_chat_id.clone().unwrap();

            for alert in &alerts {
                let msg = telegram_message(alert, &case_title);
                let token_clone = token.clone();
                let chat_id_clone = chat_id.clone();
                retry_send(move || {
                    let token = token_clone.clone();
                    let chat_id = chat_id_clone.clone();
                    let msg = msg.clone();
                    async move {
                        let client = reqwest::Client::new();
                        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                        client.post(&url)
                            .json(&serde_json::json!({
                                "chat_id": chat_id,
                                "text": msg,
                                "parse_mode": "HTML",
                            }))
                            .timeout(Duration::from_secs(10))
                            .send()
                            .await
                    }
                }, 3).await;
            }
        }

        tracing::info!(
            case_id = %case_id,
            alert_count = alerts.len(),
            "webhook notifications sent"
        );
    });
}

/// Load alerts from DB by IDs
async fn load_alerts(pool: &SqlitePool, ids: &[Uuid]) -> Result<Vec<Alert>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let mut alerts = Vec::new();
    for id in ids {
        let row: Option<(String, String, String, i64, String, String, String, String)> =
            sqlx::query_as(
                "SELECT id, case_id, pattern, score, status, severity, summary, created_at FROM alerts WHERE id = ?1",
            )
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

        if let Some((id, case_id, pattern, score, status, severity, summary, created_at)) = row {
            alerts.push(Alert {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                case_id: Uuid::parse_str(&case_id).unwrap_or_default(),
                pattern,
                severity: severity.parse().unwrap_or(Severity::Low),
                score: score as u8,
                status: status.parse().unwrap_or(crate::models::AlertStatus::Open),
                entity_ids: vec![],
                evidence_event_ids: vec![],
                summary,
                created_at,
            });
        }
    }
    Ok(alerts)
}

/// Load case title from DB
async fn load_case_title(pool: &SqlitePool, case_id: Uuid) -> Option<String> {
    sqlx::query_scalar("SELECT title FROM cases WHERE id = ?1")
        .bind(case_id.to_string())
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .flatten()
}

// ─── Discord ────────────────────────────────────────────────────────────

/// Build a Discord rich embed for an alert
fn discord_embed(alert: &Alert, case_title: &str) -> serde_json::Value {
    let (emoji, color) = severity_style(&alert.severity);
    let title = format!("{} {} — {}", emoji, alert.severity_str(), alert.pattern.replace('_', " "));

    let mut fields = vec![
        serde_json::json!({
            "name": "Case",
            "value": case_title,
            "inline": true
        }),
        serde_json::json!({
            "name": "Score",
            "value": format!("{}/100", alert.score),
            "inline": true
        }),
        serde_json::json!({
            "name": "Pattern",
            "value": alert.pattern.replace('_', " "),
            "inline": true
        }),
    ];

    if !alert.entity_ids.is_empty() {
        fields.push(serde_json::json!({
            "name": "Entities",
            "value": format!("{} linked", alert.entity_ids.len()),
            "inline": true
        }));
    }

    if !alert.evidence_event_ids.is_empty() {
        fields.push(serde_json::json!({
            "name": "Evidence",
            "value": format!("{} events", alert.evidence_event_ids.len()),
            "inline": true
        }));
    }

    let footer_text = format!("NETRA Alert • Status: {}", alert.status_str());

    serde_json::json!({
        "title": title,
        "description": alert.summary,
        "color": color,
        "fields": fields,
        "footer": { "text": footer_text },
        "timestamp": alert.created_at,
    })
}

/// Severity → Discord embed color + emoji
fn severity_style(severity: &Severity) -> (&'static str, u32) {
    match severity {
        Severity::Critical => ("🔴", 15548997),  // Red
        Severity::High => ("🟠", 15105570),      // Orange
        Severity::Medium => ("🟡", 16776960),    // Yellow
        Severity::Low => ("⚪", 9807270),        // Gray
    }
}

// ─── Telegram ───────────────────────────────────────────────────────────

/// Build a Telegram message for an alert
fn telegram_message(alert: &Alert, case_title: &str) -> String {
    let (emoji, _) = severity_style(&alert.severity);
    let pattern_display = alert.pattern.replace('_', " ");

    format!(
        "{emoji} <b>NETRA Alert — {severity}</b>\n\n\
         <b>Case:</b> {case}\n\
         <b>Pattern:</b> {pattern}\n\
         <b>Score:</b> {score}/100\n\
         <b>Status:</b> {status}\n\n\
         {summary}",
        emoji = emoji,
        severity = alert.severity_str(),
        case = escape_html(case_title),
        pattern = escape_html(&pattern_display),
        score = alert.score,
        status = alert.status_str(),
        summary = escape_html(&alert.summary),
    )
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ─── Retry Logic ────────────────────────────────────────────────────────

/// Retry an async operation with exponential backoff.
/// Returns Ok(()) if any attempt succeeds, Err after all retries exhausted.
async fn retry_send<F, Fut>(mut f: F, max_retries: u32)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
{
    let mut delay = Duration::from_millis(500);

    for attempt in 0..=max_retries {
        match f().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return; // Success
                }

                // Handle rate limiting
                if status.as_u16() == 429 {
                    let retry_after = resp
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(5);
                    tracing::warn!(
                        status = status.as_u16(),
                        retry_after = retry_after,
                        attempt = attempt + 1,
                        "webhook rate limited"
                    );
                    tokio::time::sleep(Duration::from_secs(retry_after)).await;
                    continue;
                }

                // Don't retry on client errors (4xx except 429)
                if status.is_client_error() {
                    tracing::error!(
                        status = status.as_u16(),
                        attempt = attempt + 1,
                        "webhook client error (not retrying)"
                    );
                    return;
                }

                // Server error — retry
                tracing::warn!(
                    status = status.as_u16(),
                    attempt = attempt + 1,
                    "webhook server error, retrying"
                );
            }
            Err(e) => {
                if attempt == max_retries {
                    tracing::error!(
                        err = %e,
                        attempts = attempt + 1,
                        "webhook delivery failed after all retries"
                    );
                    return;
                }
                tracing::warn!(
                    err = %e,
                    attempt = attempt + 1,
                    "webhook delivery failed, retrying"
                );
            }
        }

        tokio::time::sleep(delay).await;
        delay = delay.mul_f64(3.0); // Exponential backoff: 500ms, 1.5s, 4.5s
    }
}

// ─── Helper extensions on Alert ─────────────────────────────────────────

impl Alert {
    fn severity_str(&self) -> &'static str {
        match self.severity {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }

    fn status_str(&self) -> &'static str {
        match self.status {
            crate::models::AlertStatus::Open => "Open",
            crate::models::AlertStatus::Reviewing => "Reviewing",
            crate::models::AlertStatus::Confirmed => "Confirmed",
            crate::models::AlertStatus::FalsePositive => "False Positive",
        }
    }
}
