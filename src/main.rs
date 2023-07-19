use axum::routing::{post, Router};
use camino::Utf8Path;
use color_eyre::eyre::Result;
use secrecy::SecretString;
use std::net::SocketAddr;
use tailforward::{axumlib, handlers::post_webhook::webhook_handler};
use tap::Tap;
use tokio::fs::read_to_string;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    setup_tracing();
    setup_server().await?;
    Ok(())
}

fn setup_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .map_or_else(|_| EnvFilter::new("info"), |env_filter| env_filter);
    Registry::default()
        .with(env_filter)
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(ErrorLayer::default())
        .init();

    info!("Initialized tracing and logging systems");
}

#[tracing::instrument]
async fn setup_server() -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 33010))
        .tap(|addr| info!("Will use socket address: {}", addr));

    let tailscale_secret_path = Utf8Path::new("/secrets/tailscale-webhook");
    debug!(
        "Reading Tailscale secret from path: {}",
        tailscale_secret_path
    );
    let tailscale_secret: SecretString = read_to_string(tailscale_secret_path)
        .await?
        .tap_dbg(|val| debug!("Secret value is: {}", val))
        .into();
    info!("Read Tailscale secret from path: {}", tailscale_secret_path);

    let telegram_secret_path = Utf8Path::new("/secrets/telegram");
    debug!(
        "Reading Telegram secret from path: {}",
        telegram_secret_path
    );
    let telegram_secret: SecretString = read_to_string(telegram_secret_path)
        .await?
        .split('=')
        .collect::<Vec<_>>()[1]
        .to_string()
        .tap_dbg(|val| debug!("Secret value is: {}", val))
        .into();
    info!("Read Telegram secret from path: {}", telegram_secret_path);

    let reqwest_client = reqwest::Client::new();
    info!("Created reqwest client");

    let state = axumlib::State {
        tailscale_secret,
        telegram_secret,
        reqwest_client,
    };

    let app = Router::new()
        .fallback(axumlib::fallback)
        .route("/", post(webhook_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(axumlib::shutdown_signal())
        .await?;

    Ok(())
}
