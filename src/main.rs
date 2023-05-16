use axum::{handler::HandlerWithoutStateExt, routing::post, Router};
use color_eyre::eyre::Result;
use serde_json::{json, Value};
use std::net::SocketAddr;
use thiserror::Error;
use tokio::signal;
use tracing::{
    dispatcher::{self, Dispatch},
    info,
};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .map_or_else(|_| EnvFilter::new("info"), |env_filter| env_filter);
    let subscriber = Registry::default().with(env_filter).with(
        HierarchicalLayer::new(2)
            .with_targets(true)
            .with_bracketed_fields(true),
    );

    dispatcher::set_global_default(Dispatch::new(subscriber))?;
    info!("Initialized tracing and logging systems");

    color_eyre::install()?;
    info!("Initialized panic and error report handlers");

    let addr = SocketAddr::from(([0, 0, 0, 0], 33010));
    info!(addr = &addr.to_string(), "Will use socket address");

    let app = Router::new()
        .fallback(fallback)
        .route("/", post(webhook_handler));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[warn(clippy::redundant_pub_crate, clippy::expect_used)]
async fn shutdown_signal() {
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

struct Event {
    timestamp: String,
    version: i8,
    r#type: String,
    tailnet: String,
    message: String,
    data: String,
}

#[derive(Error, Debug)]
enum TailscaleWebhookError {
    #[error("webhook has no signature")]
    NotSigned(String),
    #[error("webhook has an invalid signature")]
    InvalidHeader(String),
}

const CURRENT_VERSION: &'static str = "v1";
const SECRET: &'static str = "tskey";

#[warn(clippy::unused_async)]
async fn webhook_handler() {
    unimplemented!();
}

#[warn(clippy::unused_async)]
async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, format!("No route {uri}"))
}

#[warn(clippy::unused_async)]
async fn verify_webhook_sig() {
    unimplemented!();
}

#[warn(clippy::unused_async)]
async fn parse_sig_header() {
    unimplemented!();
}
