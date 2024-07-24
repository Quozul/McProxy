use std::error::Error;

use crate::proxy_server::proxy_connection::proxy_connection;
use tokio::net::TcpListener;
use tracing::info;

pub(crate) async fn start_tcp_proxy(
    listen_address: &str,
    server_address: String,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&listen_address).await?;
    info!("Listening on: {}", listen_address);

    while let Ok((inbound, address)) = listener.accept().await {
        let server_address = server_address.clone();
        tokio::spawn(async move {
            proxy_connection("tcp", inbound, address, &server_address).await;
        });
    }

    Ok(())
}
