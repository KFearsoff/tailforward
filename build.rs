use camino::Utf8Path;
use std::{error::Error, fs};
use tailforward_cfg::config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = Config::default();

    let json_path = Utf8Path::new("examples/config.json");
    let json = serde_json::to_string(&cfg)?;
    fs::write(json_path, json)?;

    let toml_path = Utf8Path::new("examples/config.toml");
    let toml = toml::to_string(&cfg)?;
    fs::write(toml_path, toml)?;

    let yaml_path = Utf8Path::new("examples/config.yaml");
    let yaml = serde_yaml::to_string(&cfg)?;
    fs::write(yaml_path, yaml)?;

    Ok(())
}
