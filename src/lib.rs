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
}

mod services {
    pub mod post_webhook;
    pub mod telegram;
}

use axum::http::StatusCode;
use axum::routing::{get, post, Router};
use color_eyre::eyre::Result;
use handlers::{ping_handler, webhook_handler};
use opentelemetry::KeyValue;
use opentelemetry_sdk::{trace, Resource};
use secrecy::SecretString;
use std::fs::read_to_string;
use tap::Tap;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, warn};
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
        _ = ctrl_c => {},
        _ = terminate => {},
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
    pub tailscale_secret: SecretString,
    pub telegram_secret: SecretString,
    pub reqwest_client: reqwest::Client,
    pub chat_id: i64,
}

#[allow(clippy::missing_errors_doc)]
pub fn setup_tracing() -> Result<()> {
    // Create env filter
    let env_filter = EnvFilter::try_from_default_env()
        .map_or_else(|_| EnvFilter::new("info"), |env_filter| env_filter);

    // Install a new OpenTelemetry trace pipeline
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "tailforward",
            )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
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
pub fn setup_app(settings: &tailforward_cfg::Config) -> Result<Router> {
    let chat_id = settings.chat_id;
    let tailscale_secret_path = &settings.tailscale_secret_file;
    debug!(?tailscale_secret_path, "Reading Tailscale secret");
    let tailscale_secret: SecretString = read_to_string(tailscale_secret_path)?
        .tap_dbg(|tailscale_secret| debug!(?tailscale_secret))
        .into();
    info!(?tailscale_secret_path, "Read Tailscale secret");

    let telegram_secret_path = &settings.telegram_secret_file;
    debug!(?telegram_secret_path, "Reading Telegram secret");
    let telegram_secret: SecretString = read_to_string(telegram_secret_path)?
        .split('=')
        .collect::<Vec<_>>()[1]
        .to_string()
        .tap_dbg(|telegram_secret| debug!(?telegram_secret))
        .into();
    info!(?telegram_secret_path, "Read Telegram secret");

    let reqwest_client = reqwest::Client::new();
    info!("Created reqwest client");

    let state = State {
        tailscale_secret,
        telegram_secret,
        reqwest_client,
        chat_id,
    };

    Ok(Router::new()
        .fallback(fallback)
        .route("/tailscale-webhook", post(webhook_handler))
        .route("/ping", get(ping_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}
