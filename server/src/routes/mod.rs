pub mod alerts;
pub mod auth;
pub mod cases;
pub mod chat;
pub mod dashboard;
pub mod entities;
pub mod export;
pub mod events;
pub mod geo;
pub mod health;
pub mod ingest;
pub mod preview;
pub mod reports;
pub mod search;
pub mod settings;
pub mod users;
pub mod ws;

use axum::routing::{get, patch, post};
use axum::Router;

use crate::state::AppState;

pub fn router(state: AppState, login_limiter: std::sync::Arc<crate::ratelimit::RateLimiter>) -> Router {
    let login_limiter_layer = axum::middleware::from_fn_with_state(
        login_limiter.clone(),
        crate::ratelimit::login_rate_limit,
    );

    // Login route gets its own router with rate limiting
    let login_api = Router::new()
        .route("/auth/login", post(auth::login))
        .layer(login_limiter_layer);

    // All other routes
    let other_api = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/users", get(users::list).post(users::create))
        .route(
            "/users/{id}",
            patch(users::update).delete(users::deactivate),
        )
        .route("/cases", get(cases::list).post(cases::create))
        .route(
            "/cases/{id}",
            get(cases::detail).patch(cases::update).delete(cases::delete),
        )
        .route("/cases/{id}/audit", get(cases::audit))
        .route("/cases/{id}/events", get(events::list))
        .route("/events/{id}", get(events::detail))
        .route("/events/{id}/notes", post(events::annotate))
        .route("/cases/{id}/entities", get(entities::list))
        .route("/cases/{id}/graph", get(entities::graph))
        .route("/cases/{id}/resolve", post(entities::resolve_endpoint))
        .route(
            "/entities/{id}/profile",
            get(entities::profile),
        )
        .route("/entities/{id}", patch(entities::annotate))
        .route("/alerts", get(alerts::list))
        .route("/alerts/{id}", get(alerts::detail))
        .route("/alerts/{id}/status", patch(alerts::update_status))
        .route("/cases/{id}/analyze", post(alerts::analyze))
        .route("/cases/{id}/ingest", post(ingest::upload))
        .route("/ingest/jobs/{id}", get(ingest::job))
        .route("/cases/{id}/movements", get(crate::routes::geo::movements))
        .route("/cases/{id}/chat", post(chat::ask))
        .route("/cases/{id}/reports", post(reports::generate).get(reports::list))
        .route("/reports/{id}", get(reports::detail))
        .route("/reports/{id}/export", get(reports::export_pdf))
        .route("/reports/{id}/approve", patch(reports::approve))
        .route(
            "/settings/webhooks",
            get(settings::get_webhooks).patch(settings::update_webhooks),
        )
        .route("/models", get(settings::models))
        .route("/models/promote", post(settings::promote_model))
        .route("/training/trigger", post(settings::trigger_training))
        .route("/training/queue", get(settings::queue))
        .route("/settings/alerts", get(settings::get_alert_thresholds).patch(settings::update_alert_thresholds))
        .route("/settings/retention", get(settings::get_retention).patch(settings::update_retention))
        .route("/dashboard", get(dashboard::dashboard))
        .route("/search", get(search::search))
        .route("/cases/{id}/export", get(export::export_case))
        .route("/cases/{id}/ingest/preview", post(preview::preview));

    let api = login_api.merge(other_api);

    Router::new()
        .route("/health", get(health::health))
        .route("/ws", get(ws::handler))
        .nest("/api/v1", api)
        .with_state(state)
}
