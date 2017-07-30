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
use config::Config;

mod config;

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

pub use errors::*;

pub fn run() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .author(crate_authors!())
        .version(crate_version!())
        .get_matches();
    let config = Config::new(matches.value_of("config"))
        .chain_err(|| "Unable to read config")?;

    let verbose = matches.occurrences_of("verbose");

    if verbose > 1 {
        println!("Configuration:\n{:#?}", config);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn runs_ok() {
        run().unwrap();
    }
}
