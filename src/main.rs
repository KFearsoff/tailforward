use axum::routing::{post, Router};
use color_eyre::eyre::Result;
use std::net::SocketAddr;
use tracing::{
    dispatcher::{self, Dispatch},
    info,
};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;
use ts_to_tg::{axumlib, handle};

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
        .fallback(axumlib::fallback)
        .route("/", post(handle::webhook_handler));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(axumlib::shutdown_signal())
        .await?;

    Ok(())
}
