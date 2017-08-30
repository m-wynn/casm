use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use xdg::BaseDirectories;
use toml;
pub use errors::*;

#[derive(Deserialize, Debug, PartialEq)]
/// The user-specified config
pub struct Config {
    /// The source folder that all files are relative to (i.e. `~/Music/`)
    pub source_folder: String,
    /// The dest folder to which all relative filenames will be appended (i.e.
    /// `/mnt/Internal_Storage/Music`)
    pub dest_folder: String,
    /// The regex that matches files to exclude, usually for instrumental tracks
    pub exclude: Option<Vec<String>>,
    /// A list of files, folders, and glob patterns to convert (this will be unique'd later)
    pub files: Vec<String>,
    /// Conversion-specific settings
    pub convert_profile: ConvertProfile,
}

// Conversion options
#[derive(Deserialize, Debug, PartialEq)]
/// The conversion-specific parts of the user-specified config
pub struct ConvertProfile {
    /// A `ffmpeg::codec::id::Id` that is supported (see gen_codec_types).  Files that are not
    /// already one of `acceptable_formats` will be converted to this format.
    pub target_format: String,
    /// A list of music types that are acceptable on the target.  If the source file is one of
    /// these, it will not be converted to target_format.  The list may include types:
    /// * `ffmpeg::codec::id::Id`s that are supported (see gen_codec_types)
    /// * `lossless` for all lossless files (not recommended, unless also combined with lossy)
    /// * `lossy` for all lossy files (recommended)
    /// I usually go with `lossy` here, so that I'm not converting between lossy formats (more
    /// loss!) but don't have giant lossy files on my phone.
    pub acceptable_formats: Vec<String>,
    /// A target bit rate in KB/s (i.e. 320 or 128)
    pub bit_rate: usize,
}


impl Config {
    /// Creates a config struct from the configuration file
    ///
    /// # Arguments
    ///
    /// * `config_cli` - An Option that may hold the location of the configuration file.
    /// Otherwise, the xdg location will be used.
    pub fn new(config_cli: Option<&str>) -> Result<Config> {
        let config_file = match config_cli {
            Some(file) => PathBuf::from(file),
            _ => {
                let xdg_dirs = BaseDirectories::with_prefix("casm").chain_err(
                    || "Unable to get XDG directories",
                )?;
                if let Some(file) = xdg_dirs.find_config_file("config.toml") {
                    file
                } else {
                    bail!("Could not load config.toml")
                }
            }
        };

        let mut config_file = File::open(config_file).chain_err(
            || "Unable to open config file",
        )?;
        let mut contents = String::new();
        config_file.read_to_string(&mut contents).chain_err(
            || "Unable to read config file",
        )?;

        // TODO: Allow overriding by CLI args
        let config = toml::from_str(contents.as_str()).chain_err(
            || "Could not parse config file",
        )?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, ConvertProfile};

    #[test]
    fn invalid_config_path_err() {
        assert!(Config::new(Some("/tmp/does_not_exist")).is_err())
    }

    #[test]
    fn valid_config_path() {
        let correct_config = Config {
            source_folder: "/home/matthew/Music".to_owned(),
            dest_folder: "/home/matthew/mnt/Internal Storage/Music".to_owned(),
            exclude: Some(vec![
                r".*[Ii]nstrument(al)?( ver(.?|sion))?(\)|-|>)?\.[a-zA-Z0-9]+$".to_owned(),
                r".*[O|o]ff-?[V|v]ocal.*".to_owned(),
            ]),
            files: vec!["BLACKPINK".to_owned(), "MAMAMOO".to_owned()],
            convert_profile: ConvertProfile {
                target_format: "OPUS".to_owned(),
                acceptable_formats: vec!["quality:lossy".to_owned()],
                bit_rate: 320,
            },
        };
        assert_eq!(
            Config::new(Some("config.example.toml")).unwrap(),
            correct_config
        )
    }
}
