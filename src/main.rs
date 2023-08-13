use axum::routing::{post, Router};
use color_eyre::eyre::Result;
use secrecy::SecretString;
use tailforward::{axumlib, config::new_config, handlers::webhook_handler};
use tap::Tap;
use tokio::fs::read_to_string;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    setup_tracing()?;
    color_eyre::install()?;
    let settings = new_config()?.tap(|settings| debug!(?settings, "Read settings"));
    setup_server(&settings).await?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

fn setup_tracing() -> Result<()> {
    // Create env filter
    let env_filter = EnvFilter::try_from_default_env()
        .map_or_else(|_| EnvFilter::new("info"), |env_filter| env_filter);

    // Install a new OpenTelemetry trace pipeline
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
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
async fn setup_server(settings: &tailforward_cfg::Config) -> Result<()> {
    let addr = settings.address;
    let chat_id = settings.chat_id;
    let tailscale_secret_path = &settings.tailscale_secret_file;
    debug!(?tailscale_secret_path, "Reading Tailscale secret");
    let tailscale_secret: SecretString = read_to_string(tailscale_secret_path)
        .await?
        .tap_dbg(|tailscale_secret| debug!(?tailscale_secret))
        .into();
    info!(?tailscale_secret_path, "Read Tailscale secret");

    let telegram_secret_path = &settings.telegram_secret_file;
    debug!(?telegram_secret_path, "Reading Telegram secret");
    let telegram_secret: SecretString = read_to_string(telegram_secret_path)
        .await?
        .split('=')
        .collect::<Vec<_>>()[1]
        .to_string()
        .tap_dbg(|telegram_secret| debug!(?telegram_secret))
        .into();
    info!(?telegram_secret_path, "Read Telegram secret");

    let reqwest_client = reqwest::Client::new();
    info!("Created reqwest client");

    let state = axumlib::State {
        tailscale_secret,
        telegram_secret,
        reqwest_client,
        chat_id,
    };

    let app = Router::new()
        .fallback(axumlib::fallback)
        .route("/tailscale-webhook", post(webhook_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(axumlib::shutdown_signal())
        .await?;

    Ok(())
}
