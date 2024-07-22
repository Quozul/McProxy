use std::error::Error;
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

use tokio::net::TcpListener;
use tracing::{debug, error, info, trace, warn};

use crate::configuration::Host;
use crate::minecraft_protocol::parse_packet::{parse_minecraft_packet, Packet};
use crate::proxy_server::minecraft::client::Client;
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
            let mut buf = vec![0; 16_384];

            let bytes_received = client.socket.peek(&mut buf).await.unwrap_or_else(|err| {
                error!("Failed to read; error={err}");
                0
            });

            if bytes_received == 0 {
                error!("No bytes received from socket");
                return;
            }

            if !client.is_handshaking() {
                error!("The client is not in an handshaking state");
                return;
            }

            let hostname = handshake_client(&buf.clone(), &mut client, bytes_received);
            if let Some(hostname) = hostname {
                let hosts = hosts.lock();
                let host = find_host_by_hostname(hosts, hostname.clone());

                if let Some(server_addr) = host {
                    proxy_connection("minecraft", client.socket, address, &server_addr).await;
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
