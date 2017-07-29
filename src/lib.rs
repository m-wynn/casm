#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate toml;
extern crate xdg;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use clap::App;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

pub use errors::*;

#[derive(Deserialize)]
pub struct Config {
    source_folder: String,
}

impl Config {
    pub fn new() -> Result<Config> {
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml)
            .author(crate_authors!())
            .version(crate_version!())
            .get_matches();
        let config_file = match matches.value_of("config") {
            Some(file) => { PathBuf::from(file) },
            _ => {
                let xdg_dirs = xdg::BaseDirectories::with_prefix("casm")
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

pub fn run() -> Result<()> {
    let config = Config::new()
        .chain_err(|| "Unable to read config")?;
    Ok(())
}
