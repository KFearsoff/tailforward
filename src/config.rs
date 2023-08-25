use color_eyre::Result;
use config::{Config, Environment};
use secrecy::SecretString;
use std::fs::read_to_string;
use tap::Tap;
use tracing::{debug, info};

#[tracing::instrument]
pub fn new_config() -> Result<Application> {
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
    let base: tailforward_cfg::Config = s.try_deserialize()?;

    let tailscale_secret_path = &base.tailscale_secret_file;
    debug!(?tailscale_secret_path, "Reading Tailscale secret");
    let tailscale_secret: SecretString = read_to_string(tailscale_secret_path)?
        .tap_dbg(|tailscale_secret| debug!(?tailscale_secret))
        .into();
    info!(?tailscale_secret_path, "Read Tailscale secret");

    let telegram_secret_path = &base.telegram_secret_file;
    debug!(?telegram_secret_path, "Reading Telegram secret");
    let telegram_secret: SecretString = read_to_string(telegram_secret_path)?
        .split('=')
        .collect::<Vec<_>>()[1]
        .to_string()
        .tap_dbg(|telegram_secret| debug!(?telegram_secret))
        .into();
    info!(?telegram_secret_path, "Read Telegram secret");

    Ok(Application {
        base,
        tailscale_secret,
        telegram_secret,
    })
}

#[tracing::instrument]
pub fn new_config_with_secrets(
    tailscale_secret: SecretString,
    telegram_secret: SecretString,
) -> Result<Application> {
    let s = Config::builder()
        .add_source(Environment::with_prefix("tailforward"))
        .build()?;

    let base = s.try_deserialize()?;
    Ok(Application {
        base,
        tailscale_secret,
        telegram_secret,
    })
}

#[derive(Clone, Debug)]
pub struct Application {
    pub base: tailforward_cfg::Config,
    pub tailscale_secret: SecretString,
    pub telegram_secret: SecretString,
}
