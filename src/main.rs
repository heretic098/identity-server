use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router, serve,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Application state shared across request handlers
#[derive(Clone, Debug)]
struct AppState {
    // You can add fields here as needed
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    info!("Initializing API server");

    // Create shared application state
    let state = Arc::new(AppState {});

    // Create router and configure routes
    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(state);

    // Bind to the server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Starting server on {}", addr);

    // Start the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

// Health check handler
async fn health_check() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("OK"))
        .unwrap()
}

// Graceful shutdown handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM, starting graceful shutdown");
        },
    }

    // Add a small delay to allow in-flight requests to complete
    tokio::time::sleep(Duration::from_secs(1)).await;
}
