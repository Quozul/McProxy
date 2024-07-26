use std::net::SocketAddr;
use thiserror::Error;
use tokio::io::{copy_bidirectional, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{error, info};

#[derive(Error, Debug)]
pub(crate) enum ProxyConnectionError {
    #[error("initial write failed; error={0}")]
    InitialWriteFailed(std::io::Error),
    #[error("failed to transfer; error={0}")]
    FailedToTransfer(std::io::Error),
    #[error("failed to open outbound connection; error={0}")]
    FailedToOpenOutboundConnection(std::io::Error),
}

pub(crate) async fn proxy_connection(
    protocol: &str,
    inbound: &mut TcpStream,
    inbound_address: SocketAddr,
    server_addr: &str,
    initial_bytes: Option<&[u8]>,
) -> Result<(), ProxyConnectionError> {
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
                outbound
                    .write(initial_bytes)
                    .await
                    .map_err(ProxyConnectionError::InitialWriteFailed)?;
            }

            copy_bidirectional(inbound, &mut outbound)
                .await
                .map_err(ProxyConnectionError::FailedToTransfer)?;

            Ok(())
        }
        Err(err) => Err(ProxyConnectionError::FailedToOpenOutboundConnection(err)),
    }
}
