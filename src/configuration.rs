use std::fmt::Debug;
use std::fs::File;
use std::io::Read;

use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Host {
    pub(crate) hostname: String,
    pub(crate) target: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum Servers {
    Minecraft { listen: String, hosts: Vec<Host> },
    Tcp { listen: String, redirect: String },
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) servers: Vec<Servers>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ConfigError {
    #[error("error while parsing the file: {0}")]
    ParseError(String),
    #[error("cannot read file")]
    CannotRead,
    #[error("file is not found")]
    FileNotFound,
}

pub(crate) fn read_config(config_file_name: String) -> Result<Config, ConfigError> {
    let file = File::open(config_file_name);
    if let Ok(mut file) = file {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            match toml::from_str::<Config>(&contents) {
                Ok(config) => Ok(config),
                Err(err) => Err(ConfigError::ParseError(err.message().to_string())),
            }
        } else {
            Err(ConfigError::CannotRead)
        }
    } else {
        Err(ConfigError::FileNotFound)
    }
}
