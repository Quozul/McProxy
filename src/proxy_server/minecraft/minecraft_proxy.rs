use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{debug, info};

use crate::proxy_server::minecraft::client::Client;

pub(crate) async fn start_minecraft_proxy(
    addr: String,
    hosts: Arc<HashMap<String, String>>,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    while let Ok((inbound, address)) = listener.accept().await {
        debug!("Accepted new client {}:{}", address.ip(), address.port());
        let mut client = Client::new(inbound, address);
        let hosts_ref = Arc::clone(&hosts);

        tokio::spawn(async move {
            loop {
                client.read_socket().await;

                // Once the payload is complete, we can break the loop to parse the packet
                if client.is_complete() {
                    break;
                }
            }

            client.redirect_trafic(hosts_ref).await;
        });
    }

    Ok(())
}
