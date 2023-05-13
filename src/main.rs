use color_eyre::eyre::Result;
use tracing::{
    dispatcher::{self, Dispatch},
    info,
};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .map_or_else(|_| EnvFilter::new("info"), |env_filter| env_filter);
    let subscriber = Registry::default().with(env_filter).with(
        HierarchicalLayer::new(2)
            .with_targets(true)
            .with_bracketed_fields(true),
    );

    dispatcher::set_global_default(Dispatch::new(subscriber))?;

    info!("Initialized tracing and logging systems");

    println!("Hello, world!");
    Ok(())
}
