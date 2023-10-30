#![allow(clippy::expect_used)]
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub debug: bool,
    pub tailscale: Tailscale,
    pub telegram: Telegram,
    pub address: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug: false,
            tailscale: Tailscale::default(),
            telegram: Telegram::default(),
            address: SocketAddr::from_str("0.0.0.0:33010")
                .expect("Default value for config should never panic!"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Tailscale {
    pub secret_file: Option<Utf8PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Telegram {
    pub secret_file: Option<Utf8PathBuf>,
    pub file_format: Format,
    pub chat_id: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum Format {
    #[default]
    Plain,
    Alertmanager,
}
