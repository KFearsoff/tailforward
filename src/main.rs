use color_eyre::eyre::Result;
use tailforward::{config::new_config, setup_server, setup_tracing};
use tap::Tap;
use tracing::debug;

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
