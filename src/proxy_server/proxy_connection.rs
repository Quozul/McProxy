use std::net::SocketAddr;
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tracing::{error, info};

pub(crate) async fn proxy_connection(
    protocol: &str,
    mut inbound: TcpStream,
    inbound_address: SocketAddr,
    server_addr: &str,
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
            let _ = copy_bidirectional(&mut inbound, &mut outbound)
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
