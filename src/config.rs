use color_eyre::Result;
use config::{Config, Environment};

#[tracing::instrument]
pub fn new_config() -> Result<tailforward_cfg::Config> {
    // let config_file_path = env::var("TAILFORWARD_CONFIG_FILE").unwrap_or_else(|error| {
    //     info!("TAILFORWARD_CONFIG_FILE is not specified, using defaults");
    //     debug!(?error);
    //     "examples/config.toml".to_string()
    // });

    let s = Config::builder()
        // .add_source(File::with_name(&config_file_path))
        .add_source(Environment::with_prefix("tailforward"))
        .build()?;

    // You can deserialize (and thus freeze) the entire configuration as
    Ok(s.try_deserialize()?)
}
