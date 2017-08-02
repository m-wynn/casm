use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use xdg::BaseDirectories;
use toml as toml;
pub use errors::*;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    pub source_folder: String,
    pub dest_folder: String,
    pub exclude: Option<Vec<String>>,
    pub files: Vec<String>,
}

impl Config {
    /// Creates a config struct
    ///
    /// # Arguments
    ///
    /// * `config_cli` - An Option that may hold the location of the configuration file
    pub fn new(config_cli: Option<&str>) -> Result<Config> {
        let config_file = match config_cli {
            Some(file) => { PathBuf::from(file) },
            _ => {
                let xdg_dirs = BaseDirectories::with_prefix("casm")
                    .chain_err(|| "Unable to get XDG directories")?;
                if let Some(file) = xdg_dirs.find_config_file("config.toml") {
                    file
                } else {
                    bail!("Could not load config.toml")
                }
            },
        };

        let mut config_file = File::open(config_file)
            .chain_err(|| "Unable to open config file")?;
        let mut contents = String::new();
        config_file.read_to_string(&mut contents)
            .chain_err(|| "Unable to read config file")?;

        // TODO: Allow overriding by CLI args
        let config = toml::from_str(contents.as_str())
            .chain_err(|| "Could not parse config file")?;

        Ok(config)
    }
}

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
            r".*[O|o]ff-?[V|v]ocal.*".to_owned()
        ]),
        files: vec![
            "Daft Punk".to_owned(),
            "MAMAMOO".to_owned()
        ],
    };
    assert_eq!(Config::new(Some("config.example.toml")).unwrap(), correct_config)
}
