use crate::configuration::Host;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::error;

mod client;
mod minecraft_proxy;
mod payload;
mod protocol;

pub(crate) fn start_minecraft_proxy(addr: String, hosts: Vec<Host>) -> JoinHandle<()> {
    let hosts = hosts
        .into_iter()
        .map(|host| (host.hostname, host.target))
        .collect::<HashMap<String, String>>();

    tokio::spawn(async move {
        let proxy = minecraft_proxy::listen(addr, Arc::new(hosts)).await;
        if let Err(err) = proxy {
            error!("error with Minecraft proxy; error={err}");
        }
    })
}
