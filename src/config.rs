use color_eyre::{eyre::eyre, Result};
use config::{Config, File, FileFormat};
use secrecy::SecretString;
use std::{env, fs::read_to_string};
use tap::{Pipe, Tap};
use tracing::{debug, info};

#[tracing::instrument]
pub fn new_config() -> Result<Application> {
    let config_dir_path = env::var("CONFIGURATION_DIRECTORY").unwrap_or_else(|error| {
        info!("CONFIGURATION_DIRECTORY is not specified, using defaults");
        debug!(?error);
        "/etc/tailforward".to_string()
    });
    let config_file_path = config_dir_path + "/tailforward";

    let s = Config::builder()
        .add_source(File::new(&config_file_path, FileFormat::Toml))
        .build()?;

    // You can deserialize (and thus freeze) the entire configuration as
    let base: tailforward_cfg::Config = s.try_deserialize()?;

    if base.telegram.chat_id.is_none() {
        return Err(eyre!("Chat id is not specified"));
    };

    let tailscale_secret_path = &base
        .tailscale
        .secret_file
        .as_ref()
        .ok_or(eyre!("Must specify path for Tailscale secret"))?;
    debug!(?tailscale_secret_path, "Reading Tailscale secret");
    let tailscale_secret: SecretString = read_to_string(tailscale_secret_path)?
        .trim()
        .to_owned()
        .tap_dbg(|tailscale_secret| debug!(?tailscale_secret))
        .into();
    info!(?tailscale_secret_path, "Read Tailscale secret");

    let telegram_secret_path = &base
        .telegram
        .secret_file
        .as_ref()
        .ok_or(eyre!("Must specify path for Telegram secret"))?;
    debug!(?telegram_secret_path, "Reading Telegram secret");
    let telegram_secret: SecretString = read_to_string(telegram_secret_path)?
        .trim()
        .to_owned()
        .pipe_as_mut(|str| match &base.telegram.file_format {
            tailforward_cfg::config::Format::Alertmanager => {
                debug!("alertmanager match");
                str.split('=').collect::<Vec<_>>()[1]
            }
            tailforward_cfg::config::Format::Plain => {
                debug!("plain match");
                str
            }
        })
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
    let s = Config::builder().build()?;

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
