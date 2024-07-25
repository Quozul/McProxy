use crate::minecraft_protocol::parse_packet::{parse_minecraft_packet, Packet};
use crate::minecraft_protocol::state::State;
use crate::proxy_server::minecraft::payload::Payload;
use crate::proxy_server::proxy_connection::proxy_connection;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::{debug, error, trace, warn};

pub(crate) struct Client {
    socket: TcpStream,
    state: State,
    payload: Payload,
    address: SocketAddr,
}

impl Client {
    pub(crate) fn new(socket: TcpStream, address: SocketAddr) -> Client {
        Client {
            socket,
            address,
            state: State::Handshake,
            payload: Payload::new(),
        }
    }

    pub(crate) fn update_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    pub(crate) fn is_handshaking(&self) -> bool {
        self.state == State::Handshake
    }

    pub(crate) async fn read_socket(&mut self) {
        let mut buf = vec![0; self.payload.get_remaining_to_read()];

        let bytes_received = self.socket.read(&mut buf).await.unwrap_or_else(|err| {
            error!("Failed to read; error={err}");
            0
        });

        if bytes_received == 0 {
            error!("No bytes received from socket");
            return;
        }

        trace!(
            "Received raw buffer({}): {}",
            bytes_received,
            print_bytes_hex(&buf.clone(), bytes_received)
        );

        if !self.is_handshaking() {
            error!("The client is not in an handshaking state");
            return;
        }

        if let Err(err) = self
            .payload
            .append_bytes(&buf[..bytes_received], bytes_received)
        {
            error!("Invalid packet; error={err}");
        }
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.payload.is_complete()
    }

    pub(crate) async fn redirect_trafic(&mut self, hosts_ref: Arc<HashMap<String, String>>) {
        if let Some(hostname) = self.get_hostname_from_payload() {
            let host = hosts_ref.get(&hostname);

            if let Some(server_addr) = host {
                proxy_connection(
                    "minecraft",
                    &mut self.socket,
                    self.address,
                    server_addr,
                    Some(self.payload.get_all_bytes()),
                )
                .await;
            } else {
                warn!("Client trying to connect to unknown server host {hostname}");
            }
        }
    }

    fn get_hostname_from_payload(&mut self) -> Option<String> {
        let bytes = self.payload.get_data();
        let length = self.payload.get_packet_size();
        trace!(
            "Received packet({}) to decode: {}",
            length,
            print_bytes_hex(bytes, length)
        );

        match parse_minecraft_packet(bytes) {
            Ok(packet) => {
                debug!("Received {}", packet);

                match packet {
                    Packet::Handshake {
                        hostname,
                        next_state,
                        ..
                    } => {
                        self.update_state(next_state);
                        Some(hostname)
                    }
                }
            }
            Err(err) => {
                warn!("Could not parse Minecraft packet: {err}");
                None
            }
        }
    }
}

fn print_bytes_hex(bytes: &[u8], length: usize) -> String {
    bytes[..length]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}
