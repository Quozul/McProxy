use std::error::Error;

use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

pub(crate) async fn start_tcp_proxy(
    listen_address: String,
    server_address: String,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&listen_address).await?;
    info!("Listening on: {}", listen_address);

    while let Ok((mut inbound, address)) = listener.accept().await {
        info!(
            "tcp:connection from {}:{} forwarded to {}",
            address.ip(),
            address.port(),
            server_address,
        );

        let mut outbound = TcpStream::connect(server_address.clone()).await?;

        tokio::spawn(async move {
            let _ = copy_bidirectional(&mut inbound, &mut outbound)
                .await
                .map_err(|err| {
                    error!("Failed to transfer; error={err}");
                });
        });
    }

    Ok(())
}
