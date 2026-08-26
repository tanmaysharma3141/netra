use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{ConnectInfo, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tokio::sync::RwLock;

/// Sliding window rate limiter: max requests per window per key.
pub struct RateLimiter {
    inner: RwLock<HashMap<String, Vec<Instant>>>,
    max_requests: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }

    /// Returns true if the request is allowed, false if rate limited.
    pub async fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);
        let mut map = self.inner.write().await;
        let timestamps = map.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove entries outside the window
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= self.max_requests as usize {
            return false;
        }

        timestamps.push(now);
        true
    }

    /// Get the number of seconds until the oldest request in the window expires.
    pub async fn retry_after(&self, key: &str) -> u64 {
        let map = self.inner.read().await;
        if let Some(timestamps) = map.get(key) {
            if let Some(oldest) = timestamps.first() {
                let window = std::time::Duration::from_secs(self.window_secs);
                let elapsed = oldest.elapsed();
                if elapsed < window {
                    return (window - elapsed).as_secs() + 1;
                }
            }
        }
        self.window_secs
    }
}

/// Axum middleware that enforces rate limiting on the login endpoint.
pub async fn login_rate_limit(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    axum::extract::State(limiter): axum::extract::State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let ip = addr.ip().to_string();
    if limiter.check(&ip).await {
        next.run(request).await
    } else {
        let retry = limiter.retry_after(&ip).await;
        tracing::warn!(ip = %ip, "rate limited on login");
        (
            StatusCode::TOO_MANY_REQUESTS,
            [(
                axum::http::header::RETRY_AFTER,
                retry.to_string().parse().unwrap_or_else(|_| axum::http::HeaderValue::from_static("60")),
            )],
            format!(
                "{{\"error\":{{\"code\":\"rate_limited\",\"message\":\"too many login attempts; retry in {retry}s\"}}}}"
            ),
        )
            .into_response()
    }
}
