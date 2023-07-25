use camino::Utf8PathBuf;
use color_eyre::Result;
use config::{Config, Environment};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub tailscale_secret_file: Utf8PathBuf,
    pub telegram_secret_file: Utf8PathBuf,
    pub address: SocketAddr,
    pub chat_id: i64,
}

impl Settings {
    #[tracing::instrument]
    pub fn new() -> Result<Self> {
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
}
