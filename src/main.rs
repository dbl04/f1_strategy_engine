//! Executable binary entrypoint for the F1 Strategy Engine HTTP web server.

use f1_strategy_engine::api::create_router;
use f1_strategy_engine::config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Executable main function initializing telemetry logging, loading server configuration,
/// binding TCP socket listener, and starting the Axum HTTP service.
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "f1_strategy_engine=info,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let addr = format!("{}:{}", config.host, config.port);

    let app = create_router();

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("🏎️ Strategy Engine running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
