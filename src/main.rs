use std::error::Error;

use clap::Parser;
use tracing::error;

use crate::backends::minecraft::start_minecraft_proxy;
use backends::tcp::start_tcp_proxy;
use configuration::{read_config, Servers};
use logging::enable_logging;

mod backends;
mod configuration;
mod logging;

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
                Servers::Minecraft { listen, hosts } => start_minecraft_proxy(listen, hosts),
                Servers::Tcp { listen, redirect } => start_tcp_proxy(listen, redirect),
            });

            futures::future::join_all(servers).await;
        }
        Err(err) => {
            error!("error while reading configuration; error={err}");
        }
    }

    Ok(())
}
