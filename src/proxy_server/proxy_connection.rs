use std::net::SocketAddr;
use tokio::io::{copy_bidirectional, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{error, info};

pub(crate) async fn proxy_connection(
    protocol: &str,
    inbound: &mut TcpStream,
    inbound_address: SocketAddr,
    server_addr: &str,
    initial_bytes: Option<&[u8]>,
) {
    info!(
        "{}:connection from {}:{} forwarded to {}",
        protocol,
        inbound_address.ip(),
        inbound_address.port(),
        server_addr,
    );
    match TcpStream::connect(server_addr).await {
        Ok(mut outbound) => {
            if let Some(initial_bytes) = initial_bytes {
                let _ = outbound.write(initial_bytes).await;
            }
            let _ = copy_bidirectional(inbound, &mut outbound)
                .await
                .map_err(|err| {
                    error!("Failed to transfer; error={err}");
                });
        }
        Err(err) => {
            error!("Failed to open outbound connection; error={err}")
        }
    }
}
