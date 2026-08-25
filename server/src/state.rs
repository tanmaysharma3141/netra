use sqlx::SqlitePool;
use tokio::sync::broadcast;

use crate::models::{WsEnvelope, WsEvent};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub tx: broadcast::Sender<WsEnvelope>,
    pub jwt_secret: std::sync::Arc<String>,
}

impl AppState {
    pub fn new(pool: SqlitePool, jwt_secret: String) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            pool,
            tx,
            jwt_secret: std::sync::Arc::new(jwt_secret),
        }
    }

    pub fn publish(&self, topic: impl Into<String>, event: WsEvent) {
        let _ = self.tx.send(WsEnvelope {
            topic: topic.into(),
            event,
        });
    }
}
