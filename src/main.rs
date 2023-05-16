use axum::routing::{post, Router};
use color_eyre::eyre::Result;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{
    dispatcher::{self, Dispatch},
    info,
};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;
use ts_to_tg::{axumlib, handle};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    setup_tracing()?;
    setup_server().await?;
    Ok(())
}

fn setup_tracing() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .map_or_else(|_| EnvFilter::new("info"), |env_filter| env_filter);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(ErrorLayer::default());

    dispatcher::set_global_default(Dispatch::new(subscriber))?;
    info!("Initialized tracing and logging systems");

    Ok(())
}

async fn setup_server() -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 33010));
    info!(addr = &addr.to_string(), "Will use socket address");

    let app = Router::new()
        .fallback(axumlib::fallback)
        .route("/", post(handle::webhook_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        );

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(axumlib::shutdown_signal())
        .await?;

    Ok(())
}
