#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate ffmpeg;
extern crate gag;
extern crate glob;
extern crate phf;
extern crate pbr;
extern crate regex;
extern crate serde;
extern crate scoped_threadpool;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate unicase;
extern crate walkdir;
extern crate xdg;

use clap::App;
use config::Config;
use glob::glob;
use musicfile::Musicfile;
use pbr::ProgressBar;
use regex::RegexSet;
use scoped_threadpool::Pool;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use unicase::UniCase;
use walkdir::WalkDir;

include!("codecs_generated.rs");

mod codec;
mod config;
mod musicfile;
mod transcoder;

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
    let config = Config::new(matches.value_of("config")).chain_err(
        || "Unable to read config",
    )?;

    let verbose = matches.occurrences_of("verbose");

    if verbose > 1 {
        println!("Configuration:\n{:#?}", config);
    }

    let exclude = match config.exclude {
        Some(exclude_pattern) => {
            Some(RegexSet::new(exclude_pattern).chain_err(
                || "Config exclude is not valid regex",
            )?)
        }
        _ => None,
    };

    let files = scan_files(&config.source_folder, config.files, &exclude);

    if verbose > 2 {
        println!("Files:\n{:#?}", files);
    }

    process_files(
        files,
        &config.source_folder,
        &config.dest_folder,
        &config.convert_profile,
    );

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
                Ok(path) => {
                    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                        if let Some(musicfile) = Musicfile::new(
                            entry.path().to_path_buf(),
                            exclude,
                        )
                        {
                            musicfiles.insert(musicfile);
                        }
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }
    musicfiles
}

/// Processes each file
fn process_files(
    musicfiles: HashSet<Musicfile>,
    source_folder: &str,
    dest_folder: &str,
    convert_profile: &config::ConvertProfile,
) {
    let mut pool = Pool::new(4);


    let mut pb = ProgressBar::new(musicfiles.len() as u64);
    pb.tick_format("▏▎▍▌▋▊▉██▉▊▋▌▍▎▏");
    pb.show_message = true;
    let pb = Arc::new(Mutex::new(pb));

    pool.scoped(|scope| {
        for file in musicfiles {
            let pb = pb.clone();
            scope.execute(move || {
                pb.lock().unwrap().message(&format!(
                    "Processing {}: ",
                    file.filename.file_name().unwrap().to_str().unwrap_or(
                        "invalid filename",
                    )
                ));
                if let Err(ref e) = file.process_file(source_folder, dest_folder, convert_profile) {
                    use std::io::Write;
                    let stderr = &mut ::std::io::stderr();
                    let errmsg = "Error writing to stderr";
                    writeln!(
                        stderr,
                        "error processing {}: {}",
                        file.filename.to_str().unwrap_or("invalid filename"),
                        e
                    ).expect(errmsg);

                    for e in e.iter().skip(1) {
                        writeln!(stderr, "\tcaused by: {}", e).expect(errmsg);
                    }

                    // The backtrace is not always generated. Try to run this example
                    // with `RUST_BACKTRACE=1`.
                    if let Some(backtrace) = e.backtrace() {
                        writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
                    }
                }
                pb.lock().unwrap().inc();
            });
        }
    });
    pb.lock().unwrap().finish_print("Pull complete");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::{Musicfile, scan_files};

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
        let files = vec![
            "folder1/How Doth The Little Crocodile.mp3".to_owned(),
            "folder1/".to_owned(),
            "folder1/*".to_owned(),
        ];
        let musicfiles = scan_files("test-files", files, &None);
        let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
        let should_contain = Musicfile { filename: filename };
        assert_eq!(musicfiles.contains(&should_contain), true);
        assert_eq!(musicfiles.len(), 1);
    }
}
