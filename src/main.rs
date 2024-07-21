use std::error::Error;
use std::sync::{Arc, Mutex};

use clap::Parser;
use tracing::{error, Level};

use proxy_server::minecraft::minecraft_proxy::start_minecraft_proxy;

use crate::configuration::{read_config, Servers};
use crate::proxy_server::tcp::tcp_proxy::start_tcp_proxy;

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
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Cli::parse();
    let config = read_config(args.config);

    match config {
        Ok(config) => {
            let servers = config.servers.iter().cloned().map(|server| match server {
                Servers::Minecraft { listen, hosts } => {
                    let hosts = Arc::new(Mutex::new(hosts));
                    tokio::spawn(async move {
                        let _ = start_minecraft_proxy(listen, hosts).await;
                    })
                }
                Servers::Tcp { listen, redirect } => tokio::spawn(async move {
                    let _ = start_tcp_proxy(listen, redirect).await;
                }),
            });

            futures::future::join_all(servers).await;
        }
        Err(err) => {
            error!("Error while reading configuration: {err}");
        }
    }

    Ok(())
}
