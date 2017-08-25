extern crate ffmpeg;
extern crate mime_guess;

pub use errors::*;

use config;
use ffmpeg::codec;
use gag::Gag;
use regex::RegexSet;
use std::fs;
use std::path::PathBuf;
use transcoder;
use unicase::UniCase;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Musicfile {
    pub filename: PathBuf,
}

impl Musicfile {
    pub fn new(filename: PathBuf, exclude: &Option<RegexSet>) -> Option<Musicfile> {
        if mime_guess::guess_mime_type(&filename).type_() == "audio" {
            if let Some(ref exclude) = *exclude {
                if exclude.is_match(filename.to_str().unwrap_or("")) {
                    return None;
                }
            }
            return Some(Musicfile { filename: filename });
        }
        None
    }

    pub fn process_file(
        &self,
        src: &str,
        dest: &str,
        convert_profile: &config::ConvertProfile,
    ) -> Result<()> {
        let codec = self.get_codec().ok_or("Failed to get codec")?;
        let codec_info = ::ALL_CODECS.get(&UniCase(codec.name())).ok_or(
            "Not an acceptable music file",
        )?;
        // This transmute should be safe as `get` will not store the reference with
        // the expanded lifetime. This is due to `Borrow` being overly strict and
        // can't have an impl for `&'static str` to `Borrow<&'a str>`.
        let key: &str = &convert_profile.target_format.to_string();
        let key = unsafe { ::std::mem::transmute::<_, &'static str>(key) };
        let target_codec = ::ALL_CODECS.get(&UniCase(key)).ok_or(
            "Not an acceptable target format",
        )?;
        let dest_prefix = PathBuf::from(dest).join(self.filename.strip_prefix(src).chain_err(
            || "Could not strip prefix from filename",
        )?);
        fs::create_dir_all(&dest_prefix.parent().ok_or(
            "Cannot get parent of root or prefix",
        )?).chain_err(|| "Could not create destination")?;
        if codec_info.is_acceptable(&convert_profile.acceptable_formats) {
            let dest = dest_prefix.with_extension(codec_info.extension);
            if self.should_write(&dest) {
                fs::copy(&self.filename, dest).chain_err(
                    || "Could not copy file",
                )?;
            }
        } else {
            let dest = dest_prefix.with_extension(target_codec.extension);
            // ffmpeg likes writing things to stderr
            #[allow(unused)]
            let gag_stderr = Gag::stderr().unwrap();
            ffmpeg::init().unwrap();
            transcoder::convert(
                self.filename.to_str().ok_or("Invalid filename")?,
                dest.to_str().ok_or("Invalid destination")?,
                "anull",
                convert_profile.bit_rate * 1024,
            );
        }
        Ok(())
    }

    fn get_codec(&self) -> Option<codec::id::Id> {
        #[allow(unused)]
        let gag_stderr = Gag::stderr().unwrap();
        ffmpeg::init().unwrap();
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

    fn should_write(&self, dest: &PathBuf) -> bool {
        //TODO: Compare timestamps
        !dest.exists()
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

#[test]
fn test_get_codec() {
    ffmpeg::init().unwrap();
    let musicfile = Musicfile {
        filename: PathBuf::from("test-files/folder1/How Doth The Little Crocodile.mp3"),
    };
    let expected_codec = ffmpeg::codec::Id::MP3;
    assert_eq!(musicfile.get_codec(), Some(expected_codec));
}
