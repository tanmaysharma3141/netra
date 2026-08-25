use futures_util::{SinkExt, StreamExt};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;

use crate::state::AppState;

pub async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
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
