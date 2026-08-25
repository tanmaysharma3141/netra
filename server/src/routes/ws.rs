use futures_util::{SinkExt, StreamExt};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap};
use axum::response::Response;
use serde::Deserialize;

use crate::auth;
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

    let active: Option<i64> = sqlx::query_scalar("SELECT active FROM users WHERE id = ?1")
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);
    if active.unwrap_or(0) == 0 {
        return Err(auth::unauthorized_response("user inactive"));
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state)))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    tracing::info!("ws client connected");

    let forward = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(env) => {
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
                    tracing::info!(topics = ?v.get("topics"), "ws subscribe");
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
