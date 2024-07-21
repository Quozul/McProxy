use std::error::Error;
use std::sync::{Arc, Mutex};

use clap::Parser;
use tracing::{error, Level};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use proxy_server::minecraft::minecraft_proxy::start_minecraft_proxy;

use crate::configuration::{read_config, Servers};
use crate::proxy_server::tcp::tcp_proxy::start_tcp_proxy;

mod configuration;
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let log_level = match args.verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        2 => Level::TRACE,
        _ => Level::TRACE,
    };

    let registry = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()));

    match tracing_journald::layer() {
        Ok(layer) => {
            let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
            registry.with(fmt_layer).with(layer).init();
        }
        // journald is typically available on Linux systems, but nowhere else. Portable software
        // should handle its absence gracefully.
        Err(e) => {
            let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
            registry.with(fmt_layer).init();
            error!("couldn't connect to journald: {}", e);
        }
    }

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
