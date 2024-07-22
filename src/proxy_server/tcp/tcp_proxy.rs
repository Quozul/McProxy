use std::error::Error;

use crate::proxy_server::proxy_connection::proxy_connection;
use tokio::net::TcpListener;
use tracing::info;

pub(crate) async fn start_tcp_proxy(
    listen_address: &str,
    server_address: &str,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&listen_address).await?;
    info!("Listening on: {}", listen_address);

    while let Ok((inbound, address)) = listener.accept().await {
        proxy_connection("tcp", inbound, address, server_address).await;
    }

    Ok(())
}
