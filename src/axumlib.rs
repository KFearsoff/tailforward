use axum::http::StatusCode;
use tokio::signal;
use tracing::{info, warn};

#[warn(clippy::unused_async, clippy::expect_used)]
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("Ctrl-C received");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
        info!("Signal is received");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Starting graceful shutdown");
}

#[warn(clippy::unused_async)]
pub async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    let status = StatusCode::NOT_FOUND;
    warn!(
        status = &status.as_str(),
        uri = uri.to_string(),
        "Failed to serve"
    );
    (status, format!("No route {uri}"))
}
