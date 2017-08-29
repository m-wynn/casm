#[derive(Debug)]
#[allow(dead_code)]
pub struct Codec<'a> {
    pub name: &'a str,
    pub lossless: bool,
    pub extension: &'a str,
}

#[allow(dead_code)]
impl<'a> Codec<'a> {
    pub fn is_acceptable(&self, acceptable_formats: &[String]) -> bool {
        let quality = format!(
            "quality:{}",
            if self.lossless { "lossless" } else { "lossy" }
        );
        acceptable_formats.contains(&self.name.to_string()) ||
            acceptable_formats.contains(&quality.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Codec;

    #[test]
    fn test_acceptable_name() {
        let codec = Codec {
            name: "OPUS",
            lossless: false,
            extension: "opus",
        };
        let acceptable_formats = vec![
            "OPUS".to_owned(),
            "MP3".to_owned(),
            "quality:lossless".to_owned(),
        ];
        assert_eq!(codec.is_acceptable(&acceptable_formats), true)
    }

    #[test]
    fn test_unacceptable_name() {
        let codec = Codec {
            name: "OPUS",
            lossless: false,
            extension: "opus",
        };
        let acceptable_formats = vec![
            "VORBIS".to_owned(),
            "MP3".to_owned(),
            "quality:lossless".to_owned(),
        ];
        assert_eq!(codec.is_acceptable(&acceptable_formats), false)
    }

    #[test]
    fn test_acceptable_type() {
        let codec = Codec {
            name: "OPUS",
            lossless: false,
            extension: "opus",
        };
        let acceptable_formats = vec![
            "VORBIS".to_owned(),
            "MP3".to_owned(),
            "quality:lossy".to_owned(),
        ];
        assert_eq!(codec.is_acceptable(&acceptable_formats), true)
    }

    #[test]
    fn test_unacceptable_type() {
        let codec = Codec {
            name: "OPUS",
            lossless: false,
            extension: "opus",
        };
        let acceptable_formats = vec![
            "VORBIS".to_owned(),
            "MP3".to_owned(),
            "quality:lossless".to_owned(),
        ];
        assert_eq!(codec.is_acceptable(&acceptable_formats), false)
    }
}
