use axum::{
    routing::{post, Router},
    Extension,
};
use color_eyre::eyre::Result;
use secrecy::SecretString;
use std::net::SocketAddr;
use std::str::FromStr;
use tailforward::{axumlib, handlers::post_webhook::webhook_handler};
use tower_http::trace::TraceLayer;
use tracing::info;
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

async fn setup_server() -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 33010));
    info!(addr = &addr.to_string(), "Will use socket address");
    let ts = SecretString::from_str("ts")?;
    //let tg = SecretString::from_str("tg")?;

    let app = Router::new()
        .fallback(axumlib::fallback)
        .route("/", post(webhook_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(ts));
    //.layer(Extension(tg));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(axumlib::shutdown_signal())
        .await?;

    Ok(())
}
