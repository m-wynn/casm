extern crate mime_guess;
extern crate regex;

use regex::RegexSet;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Musicfile {
    pub filename: PathBuf,
}

impl Musicfile {
    pub fn new(filename: PathBuf, exclude: &Option<RegexSet>) -> Option<Musicfile> {
        if mime_guess::guess_mime_type(& filename).type_() == "audio" {
            if let &Some(ref exclude) = exclude {
                if exclude.is_match(filename.to_str().unwrap()) {
                    return None;
                }
            }
            return Some(Musicfile{ filename });
        }
        None
    }
}
