#[macro_use]
extern crate clap;

use clap::App;
use std::error::Error;

pub struct Config;

impl Config {
    pub fn new() -> Result<Config, &'static str> {
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml)
            .author(crate_authors!())
            .version(crate_version!())
            .get_matches();
        Ok(Config{})
    }
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    Ok(())
}
