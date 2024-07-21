use std::error::Error;
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use crate::configuration::Host;
use crate::minecraft_protocol::parse_packet::{Packet, parse_minecraft_packet};
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
                let n = client
                    .socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                if client.is_handshaking() {
                    let hostname = handshake_client(&buf.clone(), &mut client);
                    if let Some(hostname) = hostname {
                        let hosts = hosts.lock();
                        let host = find_host_by_hostname(hosts, hostname.clone());

                        if let Some(server_addr) = host {
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

fn handshake_client(buf: &[u8], client: &mut Client) -> Option<String> {
    let packet = parse_minecraft_packet(buf);

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
