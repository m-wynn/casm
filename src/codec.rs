#[derive(Debug)]
pub struct Codec<'a> {
    pub name: &'a str,
    pub lossless: bool,
    pub extension: &'a str,
}

impl<'a> Codec<'a> {
    pub fn is_acceptable(&self, acceptable_formats: &Vec<String>) -> bool {
        let quality = format!("quality:{}",
                              match self.lossless {
                                  true => "lossless",
                                  _ => "lossy",
                              });
        acceptable_formats.contains(&self.name.to_string()) ||
        acceptable_formats.contains(&quality.to_string())
    }
}

#[test]
fn test_acceptable_name() {
    let codec = Codec {
        name: "OPUS",
        lossless: false,
        extension: "opus",
    };
    let acceptable_formats =
        vec!["OPUS".to_owned(), "MP3".to_owned(), "quality:lossless".to_owned()];
    assert_eq!(codec.is_acceptable(&acceptable_formats), true)
}

#[test]
fn test_unacceptable_name() {
    let codec = Codec {
        name: "OPUS",
        lossless: false,
        extension: "opus",
    };
    let acceptable_formats =
        vec!["VORBIS".to_owned(), "MP3".to_owned(), "quality:lossless".to_owned()];
    assert_eq!(codec.is_acceptable(&acceptable_formats), false)
}

#[test]
fn test_acceptable_type() {
    let codec = Codec {
        name: "OPUS",
        lossless: false,
        extension: "opus",
    };
    let acceptable_formats =
        vec!["VORBIS".to_owned(), "MP3".to_owned(), "quality:lossy".to_owned()];
    assert_eq!(codec.is_acceptable(&acceptable_formats), true)
}

#[test]
fn test_unacceptable_type() {
    let codec = Codec {
        name: "OPUS",
        lossless: false,
        extension: "opus",
    };
    let acceptable_formats =
        vec!["VORBIS".to_owned(), "MP3".to_owned(), "quality:lossless".to_owned()];
    assert_eq!(codec.is_acceptable(&acceptable_formats), false)
}
