mod models;
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

    let state = AppState::new();

    tokio::spawn(ticker(state.clone()));

    let app = routes::router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8420));
    tracing::info!("NETRA stub server listening on http://{addr}");
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
