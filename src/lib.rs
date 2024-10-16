pub mod config;

pub mod handlers {
    mod post_webhook;
    pub use post_webhook::webhook_handler;
    mod ping;
    pub use ping::ping_handler;
}

pub mod models {
    pub mod error;
    pub use error::TailscaleWebhook;

    pub mod event;
    pub use event::Event;

    pub mod message;
    pub use message::Message;

    pub mod report;

    pub mod tailscale_header;
    pub use tailscale_header::Header;
}

mod services {
    pub mod post_webhook;
    pub mod telegram;
}

use crate::config::Application;
use axum::http::StatusCode;
use axum::routing::{get, post, Router};
use color_eyre::eyre::Result;
use handlers::{ping_handler, webhook_handler};
use opentelemetry::trace::TracerProvider;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

#[tracing::instrument]
#[allow(clippy::expect_used, clippy::redundant_pub_crate)]
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C handler");
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
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Starting graceful shutdown");
}

#[tracing::instrument]
async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    let status = StatusCode::NOT_FOUND;
    warn!(
        %status,
        %uri,
        "Failed to serve",
    );
    (status, format!("No route {uri}"))
}

#[derive(Clone, Debug)]
pub struct State {
    pub settings: Application,
    pub reqwest_client: reqwest::Client,
}

#[allow(clippy::missing_errors_doc)]
pub fn setup_tracing() -> Result<()> {
    // Create env filter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Install a new OpenTelemetry trace pipeline
    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    let tracer = provider.tracer("tailforward");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(env_filter)
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(ErrorLayer::default())
        .with(telemetry_layer)
        .init();

    info!("Initialized tracing and logging systems");

    Ok(())
}

#[tracing::instrument]
pub fn setup_app(settings: Application) -> Result<Router> {
    let reqwest_client = reqwest::Client::new();
    info!("Created reqwest client");

    let state = State {
        settings,
        reqwest_client,
    };

    Ok(Router::new()
        .fallback(fallback)
        .route("/tailscale-webhook", post(webhook_handler))
        .route("/ping", get(ping_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}
