use std::sync::atomic::AtomicU64;

use tokio::sync::broadcast;
use uuid::Uuid;

use crate::models::{WsEnvelope, WsEvent};

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<WsEnvelope>,
    pub request_counter: std::sync::Arc<AtomicU64>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            tx,
            request_counter: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn publish(&self, topic: impl Into<String>, event: WsEvent) {
        let _ = self.tx.send(WsEnvelope {
            topic: topic.into(),
            event,
        });
    }

    pub fn new_id() -> Uuid {
        Uuid::new_v4()
    }
}
