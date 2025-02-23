use crate::backends::minecraft::payload::{Payload, PayloadAppendError};
use crate::backends::minecraft::protocol::parse_packet::{parse_minecraft_packet, Packet};
use crate::backends::minecraft::protocol::state::State;
use crate::backends::proxy_connection::{proxy_connection, ProxyConnectionError};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::{debug, error, trace};

pub(crate) struct Client {
    socket: TcpStream,
    state: State,
    payload: Payload,
    address: SocketAddr,
}

#[derive(Error, Debug)]
pub(crate) enum ClientReadError {
    #[error("invalid packet received; error={0}")]
    InvalidPacket(PayloadAppendError),
    #[error("the client is not in an handshaking state")]
    NotInHandshakingState,
    #[error("no bytes received from the client")]
    NoBytesReceived,
    #[error("failed to read socket; error={0}")]
    FailedToRead(std::io::Error),
}

#[derive(Error, Debug)]
pub(crate) enum RedirectError {
    #[error("could not parse packet; error={0}")]
    CouldNotParsePacket(Box<dyn std::error::Error>),
    #[error("client trying to connect to unknown host {0}")]
    UnknownHost(String),
    #[error("{0}")]
    ProxyError(ProxyConnectionError),
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

    pub(crate) async fn read_socket(&mut self) -> Result<(), ClientReadError> {
        let mut buf = vec![0; self.payload.get_remaining_to_read()];

        let bytes_received = self
            .socket
            .read(&mut buf)
            .await
            .map_err(ClientReadError::FailedToRead)?;

        if bytes_received == 0 {
            return Err(ClientReadError::NoBytesReceived);
        }

        trace!(
            "Received raw buffer({}): {}",
            bytes_received,
            print_bytes_hex(&buf.clone(), bytes_received)
        );

        if !self.is_handshaking() {
            return Err(ClientReadError::NotInHandshakingState);
        }

        if let Err(err) = self
            .payload
            .append_bytes(&buf[..bytes_received], bytes_received)
        {
            return Err(ClientReadError::InvalidPacket(err));
        }

        Ok(())
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.payload.is_complete()
    }

    pub(crate) async fn redirect_trafic(
        &mut self,
        hosts_ref: Arc<HashMap<String, String>>,
    ) -> Result<(), RedirectError> {
        let hostname = self.get_hostname_from_payload()?;
        let host = hosts_ref.get(&hostname);

        if let Some(server_addr) = host {
            proxy_connection(
                "minecraft",
                &mut self.socket,
                self.address,
                server_addr,
                Some(self.payload.get_all_bytes()),
            )
            .await
            .map_err(RedirectError::ProxyError)?;
        } else {
            return Err(RedirectError::UnknownHost(hostname));
        }

        Ok(())
    }

    fn get_hostname_from_payload(&mut self) -> Result<String, RedirectError> {
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
                        Ok(hostname)
                    }
                }
            }
            Err(err) => Err(RedirectError::CouldNotParsePacket(err)),
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
