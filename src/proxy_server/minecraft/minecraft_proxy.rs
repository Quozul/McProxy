use std::error::Error;
use std::sync::{Arc, LockResult, Mutex, MutexGuard};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tracing::{debug, error, info, trace, warn};

use crate::configuration::Host;
use crate::minecraft_protocol::parse_packet::{parse_minecraft_packet, Packet};
use crate::proxy_server::minecraft::client::Client;
use crate::proxy_server::minecraft::payload::Payload;
use crate::proxy_server::proxy_connection::proxy_connection;

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
            let mut payload = Payload::new();

            loop {
                let mut buf = vec![0; payload.get_remaining_to_read()];

                let bytes_received = client.socket.read(&mut buf).await.unwrap_or_else(|err| {
                    error!("Failed to read; error={err}");
                    0
                });
                trace!(
                    "Received raw buffer({}): {}",
                    bytes_received,
                    print_bytes_hex(&buf.clone(), bytes_received)
                );

                if bytes_received == 0 {
                    error!("No bytes received from socket");
                    return;
                }

                if !client.is_handshaking() {
                    error!("The client is not in an handshaking state");
                    return;
                }

                payload.append_bytes(&buf[..bytes_received], bytes_received);

                // Once the payload is complete, we can break the loop to parce the packet
                if payload.is_complete() {
                    break;
                }
            }

            let hostname =
                handshake_client(payload.get_data(), payload.get_packet_size(), &mut client);
            if let Some(hostname) = hostname {
                let hosts = hosts.lock();
                let host = find_host_by_hostname(hosts, hostname.clone());

                if let Some(server_addr) = host {
                    proxy_connection(
                        "minecraft",
                        &mut client.socket,
                        address,
                        &server_addr,
                        Some(payload.get_all_bytes()),
                    )
                    .await;
                } else {
                    error!("Client trying to connect to unknown server host {hostname}");
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

pub(crate) fn print_bytes_hex(bytes: &[u8], length: usize) -> String {
    bytes[..length]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn handshake_client(bytes: &[u8], length: usize, client: &mut Client) -> Option<String> {
    trace!(
        "Received packet({}) to decode: {}",
        length,
        print_bytes_hex(bytes, length)
    );

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
