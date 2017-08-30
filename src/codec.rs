#[derive(Debug)]
#[allow(dead_code)]
/// A codec type for storing basic metadata used for conversion later.
pub struct Codec<'a> {
    /// The name of the codec, as specified in enum ffmpeg::codec::id::Id
    pub name: &'a str,
    /// Whether the codec is lossless or lossy
    pub lossless: bool,
    /// The extension of the codec (i.e. `.mp3`)
    pub extension: &'a str,
}

#[allow(dead_code)]
impl<'a> Codec<'a> {
    /// Checks to see if the codec is one of the acceptable codecs as specified in the user
    /// configuration
    ///
    /// # Arguments
    ///
    /// * 'acceptable_formats' - The list of acceptable formats provided in the config
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
