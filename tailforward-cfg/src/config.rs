#![allow(clippy::expect_used)]
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub debug: bool,
    pub tailscale_secret_file: Utf8PathBuf,
    pub telegram_secret_file: Utf8PathBuf,
    pub address: SocketAddr,
    pub chat_id: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug: false,
            tailscale_secret_file: "/secrets/tailscale-webhook".into(),
            telegram_secret_file: "/secrets/telegram".into(),
            address: SocketAddr::from_str("0.0.0.0:33010")
                .expect("Default value for config should never panic!"),
            chat_id: -1_001_864_190_705,
        }
    }
}
