use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::sync::{Arc, Mutex};

use clap::Parser;
use serde::Deserialize;
use tracing::{error, Level};

use crate::proxy_server::minecraft_proxy::start_minecraft_proxy;

mod minecraft_protocol;
mod proxy_server;

#[derive(Deserialize, Debug, Clone)]
struct Host {
    hostname: String,
    target: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Servers {
    Minecraft { listen: String, hosts: Vec<Host> },
}

#[derive(Deserialize, Debug)]
struct Config {
    servers: Vec<Servers>,
}

fn read_config(config_file_name: String) -> Option<Config> {
    let file = File::open(&config_file_name);
    if let Ok(mut file) = file {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            match toml::from_str::<Config>(&contents) {
                Ok(config) => Some(config),
                Err(err) => {
                    error!("Cannot parse configuration file '{config_file_name}': {err}");
                    None
                }
            }
        } else {
            error!("Cannot read configuration file '{config_file_name}'");
            None
        }
    } else {
        error!("Cannot find configuration file '{config_file_name}'");
        None
    }
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::ERROR)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Cli::parse();
    let config = read_config(args.config);

    match config {
        None => {
            exit(1);
        }
        Some(config) => {
            for server in config.servers {
                match server {
                    Servers::Minecraft { listen, hosts } => {
                        // FIXME: Allow concurrent start of multiple servers
                        let hosts = Arc::new(Mutex::new(hosts));
                        start_minecraft_proxy(listen, hosts).await?;
                    }
                }
            }
        }
    }

    Ok(())
}
