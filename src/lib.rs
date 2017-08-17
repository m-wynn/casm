#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate ffmpeg;
extern crate glob;
extern crate num_cpus;
extern crate phf;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate threadpool;
extern crate toml;
extern crate unicase;
extern crate xdg;

use clap::App;
use config::Config;
use glob::glob;
use musicfile::Musicfile;
use regex::RegexSet;
use std::collections::HashSet;
use std::path::PathBuf;
use unicase::UniCase;

include!("codecs_generated.rs");

mod codec;
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

    let files = scan_files(&config.source_folder, config.files, &exclude);

    if verbose > 2 {
        println!("Files:\n{:#?}", files);
    }

    process_files(files, &config.dest_folder, &config.convert_profile);

    Ok(())
}

/// Creates a vector of files
///
/// # Arguments
///
/// * `prefix` - The name of the root directory in which files may be found
/// * `files` - An vector of folder names and/or glob patterns
/// * `exclude` - A regex to exclude
fn scan_files(prefix: &str, files: Vec<String>, exclude: &Option<RegexSet>) -> HashSet<Musicfile> {
    let mut musicfiles = HashSet::new();
    for file in files {
        let file = prefix.to_owned() + "/" + &*file;
        for entry in glob(&*file).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => check_file(path, &mut musicfiles, exclude),
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
/// * `musicfiles` - A `HashSet` to put the legitimate Musicfile in
/// * `exclude` - An exclude pattern
fn check_file(file: PathBuf, musicfiles: &mut HashSet<Musicfile>, exclude: &Option<RegexSet>) {
    if file.is_dir() {
        for entry in file.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                check_file(entry.path(), musicfiles, exclude);
            }
        }
    } else if let Some(musicfile) = Musicfile::new(file, exclude) {
        musicfiles.insert(musicfile);
    }
}

/// Processes each file
fn process_files(musicfiles: HashSet<Musicfile>,
                 prefix: &str,
                 convert_profile: &config::ConvertProfile) {
    // Eventually this will be multithreaded, so the function is simple for now.
    ffmpeg::init().unwrap();
    for file in musicfiles {
        file.process_file(prefix, convert_profile);
    }
}


#[test]
fn test_scan_folder() {
    let files = vec!["folder1".to_owned()];
    let musicfiles = scan_files("test-files", files, &None);
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let should_contain = Musicfile { filename: filename };
    assert_eq!(musicfiles.contains(&should_contain), true);
    assert_eq!(musicfiles.len(), 1);
}

#[test]
fn test_scan_glob() {
    let files = vec!["folder*/*Crocodile*".to_owned()];
    let musicfiles = scan_files("test-files", files, &None);
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let should_contain = Musicfile { filename: filename };
    assert_eq!(musicfiles.contains(&should_contain), true);
    assert_eq!(musicfiles.len(), 1);
}

#[test]
fn test_scan_filename() {
    let files = vec!["/folder1/How Doth The Little Crocodile.mp3".to_owned()];
    let musicfiles = scan_files("test-files/", files, &None);
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let should_contain = Musicfile { filename: filename };
    assert_eq!(musicfiles.contains(&should_contain), true);
    assert_eq!(musicfiles.len(), 1);
}

#[test]
fn test_scan_empty() {
    let files = vec!["folder1/*.txt".to_owned()];
    let musicfiles = scan_files("test-files", files, &None);
    assert_eq!(musicfiles.is_empty(), true);
}

#[test]
fn test_scan_nonexistant() {
    let files = vec!["not_a_folder".to_owned()];
    let musicfiles = scan_files("test-files", files, &None);
    assert_eq!(musicfiles.is_empty(), true);
}

#[test]
fn test_scan_text_file() {
    let files = vec!["folder2/notmusic.txt".to_owned()];
    let musicfiles = scan_files("test-files", files, &None);
    assert_eq!(musicfiles.is_empty(), true);
}

#[test]
fn test_scan_duplicates() {
    let files = vec!["folder1/How Doth The Little Crocodile.mp3".to_owned(),
                     "folder1/".to_owned(),
                     "folder1/*".to_owned()];
    let musicfiles = scan_files("test-files", files, &None);
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let should_contain = Musicfile { filename: filename };
    assert_eq!(musicfiles.contains(&should_contain), true);
    assert_eq!(musicfiles.len(), 1);
}
