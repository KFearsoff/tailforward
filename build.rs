use camino::Utf8Path;
use std::{error::Error, fs};
use tailforward_cfg::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = Config::default();

    let toml_path = Utf8Path::new("examples/config.toml");
    let toml = toml::to_string(&cfg)?;
    fs::write(toml_path, toml)?;

    Ok(())
}
