#[macro_use]
extern crate clap;
extern crate glob;
#[macro_use]
extern crate error_chain;
extern crate mime_guess;
extern crate num_cpus;
extern crate threadpool;
extern crate toml;
extern crate xdg;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use clap::App;
use config::Config;
use glob::glob;
use musicfile::Musicfile;
use std::path::PathBuf;

mod config;
mod musicfile;

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

    let files = process_files(config.files);

    Ok(())
}

/// Creates a vector of files
///
/// # Arguments
///
/// * `files` - An vector of folder names and/or glob patterns
fn process_files(files: Vec<String>) -> Vec<Musicfile> {
    let mut musicfiles = Vec::new();
    for file in files {
        for entry in glob(&*file).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => check_file(path, &mut musicfiles),
                Err(e) => println!("{:?}", e),
            }
        }
    }
    musicfiles
}

/// Checks a file to see if it is a folder, a music file, or something else
///
/// # Arguments
///
/// * `file` - A PathBuf to the folder or file
/// * `vector` - A Vec to put the Musicfile in, if it is a music file
fn check_file(file: PathBuf, vector: &mut Vec<Musicfile>) {
    let mut valid = false;  // Fighting the borrow checker!
    if file.is_dir() {
        for entry in file.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                check_file(entry.path(), vector);
            }
        }
    } else {
        let extension = file.extension();
        if let Some(extension) = extension {
            let extension = extension.to_str();
            if let Some(extension) = extension {
                if mime_guess::get_mime_type(extension).type_() == "audio" {
                    valid = true;
                }
            }
        }
    }
    if valid {
        vector.push(Musicfile::new(file))
    }
}
