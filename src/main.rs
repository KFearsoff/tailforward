use color_eyre::eyre::Result;
use tailforward::{config::new_config, setup_app, setup_tracing, shutdown_signal};
use tap::Tap;
use tokio::net::TcpListener;
use tracing::debug;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    setup_tracing()?;
    color_eyre::install()?;
    let settings = new_config()?.tap(|settings| debug!(?settings, "Read settings"));

    let addr = settings.base.address;
    let app = setup_app(settings)?;
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
