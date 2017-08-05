extern crate ffmpeg;
extern crate mime_guess;
extern crate phf;
extern crate unicase;
extern crate regex;

use regex::RegexSet;
use std::path::PathBuf;
use unicase::UniCase;
use ffmpeg::codec::id::Id;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Musicfile {
    pub filename: PathBuf,
}

impl Musicfile {
    pub fn new(filename: PathBuf, exclude: &Option<RegexSet>) -> Option<Musicfile> {
        if mime_guess::guess_mime_type(&filename).type_() == "audio" {
            if let Some(ref exclude) = *exclude {
                if exclude.is_match(filename.to_str().unwrap()) {
                    return None;
                }
            }
            return Some(Musicfile { filename: filename });
        }
        None
    }

    pub fn process_file(&self, prefix: &str) {
        // TODO: return an actual error to the parent process
        // But this will need to be reimplemented for multithreaded anyways
        let codec = self.get_codec().unwrap();
        println!("Processing: {} with codec {:?}",
                 self.filename.to_str().unwrap(),
                 codec);
        println!("{:#?}", ::ALL_CODECS.get(&UniCase(codec.name())));
    }

    fn get_codec(&self) -> Option<Id> {
        match ffmpeg::format::input(&self.filename) {
            Ok(context) => {
                if let Some(stream) = context.streams().best(ffmpeg::media::Type::Audio) {
                    Some(stream.codec().id())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}


#[test]
fn test_matches_exclude() {
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let exclude = Some(RegexSet::new(&[r"^.*Crocodile\.mp3$"]).unwrap());
    assert_eq!(Musicfile::new(filename, &exclude), None);
}

#[test]
fn test_not_matches_exclude() {
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let exclude = Some(RegexSet::new(&[r"^.*Alligator\.mp3$"]).unwrap());
    let expected_musicfile = Musicfile { filename: filename.clone() };
    assert_eq!(Musicfile::new(filename, &exclude), Some(expected_musicfile));
}

#[test]
fn test_no_exclude() {
    let filename = PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3");
    let exclude = None;
    let expected_musicfile = Musicfile { filename: filename.clone() };
    assert_eq!(Musicfile::new(filename, &exclude), Some(expected_musicfile));
}
