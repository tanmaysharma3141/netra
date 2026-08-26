mod anomaly;
mod auth;
mod db;
mod ingest;
mod models;
mod resolve;
mod routes;
mod state;
mod stub_data;

use std::time::Duration;

use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "netra_server=debug,tower_http=info".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/netra.db?mode=rwc".into());
    if database_url.starts_with("sqlite://data/") {
        let _ = std::fs::create_dir_all("data");
    }

    let pool = match db::init(&database_url).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(err = %e, "database init failed");
            std::process::exit(1);
        }
    };

    let jwt_secret = match std::env::var("NETRA_JWT_SECRET") {
        Ok(s) if s.len() >= 32 => s,
        Ok(_) => {
            tracing::warn!("NETRA_JWT_SECRET set but shorter than 32 chars; generating random secret (tokens won't survive restarts)");
            uuid::Uuid::new_v4().to_string() + &uuid::Uuid::new_v4().to_string()
        }
        Err(_) => {
            tracing::warn!("NETRA_JWT_SECRET not set; generating random secret (tokens won't survive restarts)");
            uuid::Uuid::new_v4().to_string() + &uuid::Uuid::new_v4().to_string()
        }
    };

    let state = AppState::new(pool, jwt_secret);

    tokio::spawn(ticker(state.clone()));

    let app = routes::router(state)
        .layer(axum::extract::DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(cors());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8420));
    tracing::info!("NETRA server listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind failed");
    axum::serve(listener, app).await.expect("server error");
}

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

fn ticker(state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            state.publish("global", models::WsEvent::AlertCreated {
                payload: stub_data::demo_alerts().into_iter().next().unwrap(),
            });
        }
    })
}
