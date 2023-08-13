#![allow(clippy::expect_used)]
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub debug: bool,
    pub tailscale_secret_file: Utf8PathBuf,
    pub telegram_secret_file: Utf8PathBuf,
    pub address: SocketAddr,
    pub chat_id: i64,
}
