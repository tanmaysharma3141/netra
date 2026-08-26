use std::collections::HashSet;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;

use crate::auth;
use crate::models::Role;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
    headers: HeaderMap,
) -> Result<Response, Response> {
    let bearer = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);

    let token = bearer
        .or(q.token)
        .ok_or_else(|| auth::unauthorized_response("missing token"))?;
    let claims = auth::verify_token(&token, &state.jwt_secret)
        .ok_or_else(|| auth::unauthorized_response("invalid token"))?;

    // Check if token has been revoked
    if auth::is_token_revoked(&state.pool, &claims.jti).await {
        return Err(auth::unauthorized_response("token revoked"));
    }

    // Check user is active
    let row: Option<(String, i64)> =
        sqlx::query_as("SELECT role, active FROM users WHERE id = ?1")
            .bind(&claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| auth::unauthorized_response("lookup failed"))?;

    let Some((role_str, active)) = row else {
        return Err(auth::unauthorized_response("user not found"));
    };
    if active == 0 {
        return Err(auth::unauthorized_response("user inactive"));
    }

    let role: Role = role_str.parse().map_err(|_| auth::unauthorized_response("bad role"))?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, role)))
}

async fn handle_socket(socket: WebSocket, state: AppState, role: Role) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Per-connection subscription set
    let subscriptions = Arc::new(Mutex::new(HashSet::new()));
    // Admin/Supervisor see everything — pre-populate with wildcard
    let sees_all = matches!(role, Role::Admin | Role::Supervisor);

    tracing::info!(role = ?role, "ws client connected");

    let subs = subscriptions.clone();
    let forward = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(env) => {
                    // Filter: admin/supervisor see all, others only see subscribed topics + global
                    if !sees_all {
                        let subs = subs.lock().await;
                        let topic = &env.topic;
                        if topic != "global" && !subs.contains(topic.as_str()) {
                            continue; // skip — client not subscribed to this topic
                        }
                    }
                    let json = serde_json::to_string(&env).unwrap_or_default();
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "ws client lagged");
                }
                Err(_) => break,
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(v) if v.get("type").and_then(|t| t.as_str()) == Some("subscribe") => {
                    if let Some(topics) = v.get("topics").and_then(|t| t.as_array()) {
                        let mut subs = subscriptions.lock().await;
                        for t in topics {
                            if let Some(s) = t.as_str() {
                                subs.insert(s.to_string());
                                tracing::info!(topic = s, "ws client subscribed");
                            }
                        }
                    }
                }
                Ok(v) if v.get("type").and_then(|t| t.as_str()) == Some("unsubscribe") => {
                    if let Some(topics) = v.get("topics").and_then(|t| t.as_array()) {
                        let mut subs = subscriptions.lock().await;
                        for t in topics {
                            if let Some(s) = t.as_str() {
                                subs.remove(s);
                                tracing::info!(topic = s, "ws client unsubscribed");
                            }
                        }
                    }
                }
                _ => tracing::debug!(raw = %text, "ws unknown frame"),
            }
        } else if matches!(msg, Message::Close(_)) {
            break;
        }
    }

    forward.abort();
    tracing::info!("ws client disconnected");
}
