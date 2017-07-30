use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use xdg::BaseDirectories;
use toml as toml;
pub use errors::*;

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Config {
    source_folder: String,
    dest_folder: String,
    files: Vec<String>,
}

impl Config {
    /// Creates a config struct
    pub fn new(config_cli: Option<&str>) -> Result<Config> {
        let config_file = match config_cli {
            Some(file) => { PathBuf::from(file) },
            _ => {
                let xdg_dirs = BaseDirectories::with_prefix("casm")
                    .chain_err(|| "Unable to get XDG directories")?;
                if let Some(file) = xdg_dirs.find_config_file("config.toml") {
                    file
                } else {
                    bail!("Could not load config.toml")
                }
            },
        };

        let mut config_file = File::open(config_file)
            .chain_err(|| "Unable to open config file")?;
        let mut contents = String::new();
        config_file.read_to_string(&mut contents)
            .chain_err(|| "Unable to read config file")?;

        // TODO: Allow overriding by CLI args
        let config = toml::from_str(contents.as_str())
            .chain_err(|| "Could not parse config file")?;

        Ok(config)
    }
}
