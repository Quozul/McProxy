use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use clap::Parser;
use tracing::error;

use proxy_server::minecraft::minecraft_proxy::start_minecraft_proxy;

use crate::configuration::{read_config, Servers};
use crate::logging::enable_logging;
use crate::proxy_server::tcp::tcp_proxy::start_tcp_proxy;

mod configuration;
mod logging;
mod minecraft_protocol;
mod proxy_server;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    enable_logging(args.verbose);

    let config = read_config(args.config);

    match config {
        Ok(config) => {
            let servers = config.servers.iter().cloned().map(|server| match server {
                Servers::Minecraft { listen, hosts } => {
                    let hosts = hosts
                        .into_iter()
                        .map(|host| (host.hostname, host.target))
                        .collect::<HashMap<String, String>>();
                    tokio::spawn(async move {
                        let proxy = start_minecraft_proxy(listen, Arc::new(hosts)).await;
                        if let Err(err) = proxy {
                            error!("Error with Minecraft proxy: {err}");
                        }
                    })
                }
                Servers::Tcp { listen, redirect } => tokio::spawn(async move {
                    let proxy = start_tcp_proxy(&listen, redirect).await;
                    if let Err(err) = proxy {
                        error!("Error with Minecraft proxy: {err}");
                    }
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
