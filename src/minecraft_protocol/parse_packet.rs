use std::error::Error;
use std::fmt;

use crate::minecraft_protocol::data_types::var_int::{read_var_int, InvalidVarIntError};
use crate::minecraft_protocol::packets::handshaking::handle_handshake;
use crate::minecraft_protocol::state::{State, UnknownStateError};

#[derive(Debug)]
pub(crate) struct UnknownPacketError {
    packet_id: u8,
}

impl fmt::Display for UnknownPacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown packet ID {:#04x}", self.packet_id)
    }
}

impl Error for UnknownPacketError {}

pub(crate) enum Packet {
    Handshake {
        protocol: i32,
        hostname: String,
        port: u16,
        next_state: State,
    },
}

pub(crate) struct PacketLengthResult {
    pub(crate) packet_start_index: usize,
    pub(crate) packet_length: usize,
}

pub(crate) fn get_packet_length(bytes: &[u8]) -> Result<PacketLengthResult, InvalidVarIntError> {
    let mut packet_start_index = 0;
    let packet_length = read_var_int(bytes, &mut packet_start_index)? as usize; // TODO: Handle invalid lengths with unit tests
    Ok(PacketLengthResult {
        packet_length,
        packet_start_index,
    })
}

pub(crate) fn parse_minecraft_packet(bytes: &[u8]) -> Result<Packet, Box<dyn Error>> {
    let packet_id = bytes[0];
    let mut index = 1;

    let packet = match packet_id {
        0x00 => {
            let handshake = handle_handshake(bytes, &mut index)?;
            let next_state = match handshake.next_state {
                0 => Ok(State::Handshake),
                1 => Ok(State::Status),
                2 => Ok(State::Login),
                3 => Ok(State::Transfer),
                _ => Err(UnknownStateError),
            }?;

            Ok(Packet::Handshake {
                protocol: handshake.protocol,
                hostname: handshake.hostname,
                port: handshake.port,
                next_state,
            })
        }
        _ => Err(UnknownPacketError { packet_id }),
    };

    Ok(packet?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_handshake() {
        // Given
        let localhost_handshake_packet = vec![
            0x00, // Packet ID
            0xff, // Packet start
            0x05, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x63, 0xdd, 0x01,
        ];

        // When
        let handshake = parse_minecraft_packet(&localhost_handshake_packet).unwrap();

        // Then
        match handshake {
            Packet::Handshake {
                protocol,
                hostname,
                port,
                next_state,
            } => {
                assert_eq!(protocol, 767);
                assert_eq!(hostname, "localhost");
                assert_eq!(port, 25565);
                assert_eq!(next_state, State::Status);
            }
        }
    }

    #[test]
    fn should_return_error_for_unknown_packet() {
        // Given
        let unknown_packet = vec![0x02, 0x01];

        // When
        let result = parse_minecraft_packet(&unknown_packet);

        // Then
        match result {
            Err(e) => {
                let unknown_packet_error = e.downcast_ref::<UnknownPacketError>().unwrap();
                assert_eq!(unknown_packet_error.packet_id, 0x02);
            }
            Ok(_) => panic!("Expected an error but got a packet"),
        }
    }
}
