use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Host {
    pub(crate) hostname: String,
    pub(crate) target: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum Servers {
    Minecraft { listen: String, hosts: Vec<Host> },
}

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) servers: Vec<Servers>,
}

#[derive(Debug)]
pub(crate) struct ConfigError {
    reason: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl Error for ConfigError {}

impl ConfigError {
    fn new(reason: String) -> Self {
        Self { reason }
    }
}

pub(crate) fn read_config(config_file_name: String) -> Result<Config, ConfigError> {
    let file = File::open(&config_file_name);
    if let Ok(mut file) = file {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            match toml::from_str::<Config>(&contents) {
                Ok(config) => Ok(config),
                Err(err) => Err(ConfigError::new(err.message().to_string())),
            }
        } else {
            Err(ConfigError::new(format!(
                "cannot read the file '{}'",
                config_file_name
            )))
        }
    } else {
        Err(ConfigError::new(format!(
            "cannot find the file '{}'",
            config_file_name
        )))
    }
}
