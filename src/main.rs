use std::error::Error;
use std::sync::{Arc, Mutex};

use clap::Parser;
use tracing::{error, Level};

use proxy_server::minecraft::minecraft_proxy::start_minecraft_proxy;

use crate::configuration::{read_config, Servers};

mod configuration;
mod minecraft_protocol;
mod proxy_server;

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
        Ok(config) => {
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
        Err(err) => {
            error!("Error while reading configuration: {err}");
        }
    }

    Ok(())
}
