use camino::Utf8Path;
use std::{error::Error, fs};
use tailforward_cfg::{
    config::{Format, Tailscale, Telegram},
    Config,
};

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = Config {
        tailscale: Tailscale {
            secret_file: Some("/etc/tailforward/tailforward.toml".into()),
        },
        telegram: Telegram {
            secret_file: Some("/secrets/telegram".into()),
            file_format: Format::Plain,
            chat_id: Some(-123),
        },
        ..Default::default()
    };

    let toml_path = Utf8Path::new("examples/config.toml");
    let toml = toml::to_string(&cfg)?;
    fs::write(toml_path, toml)?;

    Ok(())
}
