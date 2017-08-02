#[macro_use]
extern crate clap;
extern crate glob;
#[macro_use]
extern crate error_chain;
extern crate num_cpus;
extern crate regex;
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
use regex::RegexSet;
use std::collections::HashSet;
use std::path::PathBuf;

mod config;
mod musicfile;

#[allow(unknown_lints)]
#[allow(unused_doc_comment)]
pub mod errors {
    error_chain!{}
}

pub use errors::*;

pub fn run() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .author(crate_authors!())
        .version(crate_version!())
        .get_matches();
    let config = Config::new(matches.value_of("config")).chain_err(|| "Unable to read config")?;

    let verbose = matches.occurrences_of("verbose");

    if verbose > 1 {
        println!("Configuration:\n{:#?}", config);
    }

    let exclude = match config.exclude {
        Some(exclude_pattern) => {
            Some(RegexSet::new(exclude_pattern).chain_err(|| "Config exclude is not valid regex")?)
        }
        _ => None,
    };

    let files = process_files(&config.source_folder, config.files, &exclude);

    if verbose > 2 {
        println!("Files:\n{:#?}", files);
    }

    Ok(())
}

/// Creates a vector of files
///
/// # Arguments
///
/// * `files` - An vector of folder names and/or glob patterns
fn process_files(prefix: &str, files: Vec<String>, exclude: &Option<RegexSet>) -> Vec<Musicfile> {
    let mut musicfiles = Vec::new();
    let mut filename_list = HashSet::new();
    for file in files {
        let file = prefix.to_owned() + "/" + &*file;
        for entry in glob(&*file).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => check_file(path, &mut musicfiles, &mut filename_list, exclude),
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
/// * `file` - A `PathBuf` to the folder or file
/// * `vector` - A Vec to put the Musicfile in, if it is a music file
/// * `filename_list` - A list of filenames so far
/// * `exclude` - An exclude pattern
fn check_file(file: PathBuf,
              musicfiles: &mut Vec<Musicfile>,
              filename_list: &mut HashSet<PathBuf>,
              exclude: &Option<RegexSet>) {
    if file.is_dir() {
        for entry in file.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                check_file(entry.path(), musicfiles, filename_list, exclude);
            }
        }
    } else if filename_list.insert(file.clone()) {
        if let Some(musicfile) = Musicfile::new(file, exclude) {
            musicfiles.push(musicfile);
        }
    }
}

#[test]
fn test_process_folder() {
    let files = vec!["folder1".to_owned()];
    let musicfiles = process_files("test-files".to_owned(), files, None);
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let should_contain = Musicfile { filename: filename };
    assert_eq!(musicfiles.contains(&should_contain), true);
    assert_eq!(musicfiles.len(), 1);
}

#[test]
fn test_process_glob() {
    let files = vec!["folder*/*Crocodile*".to_owned()];
    let musicfiles = process_files("test-files".to_owned(), files, None);
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let should_contain = Musicfile { filename: filename };
    assert_eq!(musicfiles.contains(&should_contain), true);
    assert_eq!(musicfiles.len(), 1);
}

#[test]
fn test_process_filename() {
    let files = vec!["/folder1/How Doth The Little Crocodile.mp3".to_owned()];
    let musicfiles = process_files("test-files/".to_owned(), files, None);
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let should_contain = Musicfile { filename: filename };
    assert_eq!(musicfiles.contains(&should_contain), true);
    assert_eq!(musicfiles.len(), 1);
}

#[test]
fn test_process_empty() {
    let files = vec!["folder1/*.txt".to_owned()];
    let musicfiles = process_files("test-files".to_owned(), files, None);
    assert_eq!(musicfiles.is_empty(), true);
}

#[test]
fn test_process_nonexistant() {
    let files = vec!["not_a_folder".to_owned()];
    let musicfiles = process_files("test-files".to_owned(), files, None);
    assert_eq!(musicfiles.is_empty(), true);
}

#[test]
fn test_process_text_file() {
    let files = vec!["folder2/notmusic.txt".to_owned()];
    let musicfiles = process_files("test-files".to_owned(), files, None);
    assert_eq!(musicfiles.is_empty(), true);
}

#[test]
fn test_process_duplicates() {
    let files = vec!["folder1/How Doth The Little Crocodile.mp3".to_owned(),
                     "folder1/".to_owned(),
                     "folder1/*".to_owned()];
    let musicfiles = process_files("test-files".to_owned(), files, None);
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let should_contain = Musicfile { filename: filename };
    assert_eq!(musicfiles.contains(&should_contain), true);
    assert_eq!(musicfiles.len(), 1);
}
