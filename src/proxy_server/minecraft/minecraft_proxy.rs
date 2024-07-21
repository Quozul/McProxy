use std::error::Error;
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

use tokio::io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, trace, warn};

use crate::configuration::Host;
use crate::minecraft_protocol::parse_packet::{parse_minecraft_packet, Packet};
use crate::proxy_server::minecraft::client::Client;

pub(crate) async fn start_minecraft_proxy(
    addr: String,
    hosts: Arc<Mutex<Vec<Host>>>,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    loop {
        let (socket, address) = listener.accept().await?;
        debug!("Accepted new client {}:{}", address.ip(), address.port());
        let mut client = Client::new(socket);

        let hosts = Arc::clone(&hosts);

        tokio::spawn(async move {
            let mut buf = vec![0; 16_384];

            loop {
                let bytes_received = client
                    .socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if bytes_received == 0 {
                    return;
                }

                if client.is_handshaking() {
                    let hostname = handshake_client(&buf.clone(), &mut client, bytes_received);
                    if let Some(hostname) = hostname {
                        let hosts = hosts.lock();
                        let host = find_host_by_hostname(hosts, hostname.clone());

                        if let Some(server_addr) = host {
                            info!(
                                "minecraft:connection from {}:{} forwarded to {}",
                                address.ip(),
                                address.port(),
                                server_addr,
                            );
                            match TcpStream::connect(server_addr.clone()).await {
                                Ok(mut outbound) => {
                                    let _ = outbound.write(&buf).await.map_err(|err| {
                                        error!(
                                            "Failed to write first packet to outbound; error={err}"
                                        );
                                    });
                                    let _ = copy_bidirectional(&mut client.socket, &mut outbound)
                                        .await
                                        .map_err(|err| {
                                            error!("Failed to transfer; error={err}");
                                        });
                                }
                                Err(err) => {
                                    error!("Failed to transfer; error={err}")
                                }
                            }
                        } else {
                            error!("Client trying to connect to unknown server host {hostname}");
                        }
                    }
                }
            }
        });
    }
}

fn find_host_by_hostname(
    hosts: LockResult<MutexGuard<Vec<Host>>>,
    hostname: String,
) -> Option<String> {
    let hosts = match hosts {
        Ok(guard) => guard,
        Err(e) => {
            error!("Failed to acquire lock: {}", e);
            return None;
        }
    };

    for host in &*hosts {
        if host.hostname == hostname {
            return Some(host.target.clone());
        }
    }

    None
}

fn handshake_client(bytes: &[u8], client: &mut Client, bytes_received: usize) -> Option<String> {
    let full_packet = bytes[..bytes_received]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    trace!("Received packet: {full_packet}");

    let packet = parse_minecraft_packet(bytes);

    match packet {
        Ok(packet) => match packet {
            Packet::Handshake {
                protocol,
                hostname,
                port,
                next_state,
            } => {
                debug!(
                    "Received handshake with protocol version: {} for hostname({}): {}:{}",
                    protocol,
                    hostname.len(),
                    hostname,
                    port,
                );

                client.update_state(next_state);
                Some(hostname)
            }
        },
        Err(err) => {
            warn!("Could not parse Minecraft packet: {err}");
            None
        }
    }
}
