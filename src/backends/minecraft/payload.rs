use crate::backends::minecraft::protocol::packets::get_packet_length::{
    get_packet_length, PacketLengthParseError,
};
use thiserror::Error;

pub(crate) struct Payload {
    expected_length: Option<usize>,
    bytes_received: usize,
    bytes: Vec<u8>,
    packet_start_index: usize,
}

#[derive(Error, Debug, PartialEq)]
pub enum PayloadAppendError {
    #[error("packet length is invalid")]
    InvalidPacketLength,
}

impl Payload {
    const MAX_READ_SIZE: usize = 16_384;

    pub(crate) fn new() -> Self {
        Self {
            expected_length: None,
            bytes_received: 0,
            packet_start_index: 0,
            bytes: Vec::new(),
        }
    }

    pub(crate) fn append_bytes(
        &mut self,
        bytes: &[u8],
        length: usize,
    ) -> Result<(), PayloadAppendError> {
        self.bytes.extend_from_slice(bytes);
        self.bytes_received += length;

        if self.expected_length.is_none() {
            match get_packet_length(&self.bytes.clone()) {
                Ok(packet_length_result) => {
                    self.packet_start_index = packet_length_result.packet_start_index;
                    let total_expected_length =
                        self.packet_start_index + packet_length_result.packet_length;
                    self.expected_length = Some(total_expected_length);
                    self.bytes.reserve(total_expected_length);
                    Ok(())
                }
                Err(err) => match err {
                    PacketLengthParseError::IncompleteLength => Ok(()), // Ignored error
                    PacketLengthParseError::NegativeLength => {
                        Err(PayloadAppendError::InvalidPacketLength)
                    }
                    PacketLengthParseError::PacketTooLarge => {
                        Err(PayloadAppendError::InvalidPacketLength)
                    }
                },
            }
        } else {
            Ok(())
        }
    }

    pub(crate) fn is_complete(&self) -> bool {
        match self.expected_length {
            None => false,
            Some(expected_length) => self.bytes_received >= expected_length,
        }
    }

    pub(crate) fn get_remaining_to_read(&self) -> usize {
        match self.expected_length {
            None => Self::MAX_READ_SIZE,
            Some(expected_length) => expected_length.saturating_sub(self.bytes_received),
        }
    }

    pub(crate) fn get_data(&self) -> &[u8] {
        &self.bytes[self.packet_start_index..]
    }

    pub(crate) fn get_all_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn get_packet_size(&self) -> usize {
        match self.expected_length {
            None => 0,
            Some(expected_length) => expected_length.saturating_sub(self.packet_start_index),
        }
    }

    #[cfg(test)]
    fn get_bytes_received(&self) -> usize {
        self.bytes_received
    }

    #[cfg(test)]
    fn get_expected_length(&self) -> usize {
        self.expected_length.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_payload_should_read_all() {
        // Given
        let payload = Payload::new();

        // When / Then
        assert_eq!(payload.get_expected_length(), 0);
        assert_eq!(payload.get_remaining_to_read(), Payload::MAX_READ_SIZE);
        assert_eq!(payload.get_bytes_received(), 0);
        assert!(!payload.is_complete());
    }

    #[test]
    fn test_should_fail_with_negative_length() {
        // Given
        let mut payload = Payload::new();
        let first_bytes = vec![0xff, 0xff, 0xff, 0xff, 0x0f];

        // When
        let result = payload.append_bytes(&first_bytes, 5);

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn test_read_first_length_byte() {
        // Given
        let mut payload = Payload::new();
        let first_bytes = vec![0x10];

        // When
        payload.append_bytes(&first_bytes, 1).unwrap();

        // Then
        assert_eq!(payload.get_expected_length(), 17);
        assert_eq!(payload.get_remaining_to_read(), 16);
        assert_eq!(payload.get_bytes_received(), 1);
        assert!(!payload.is_complete());
    }

    #[test]
    fn test_read_first_length_byte_16() {
        // Given
        let mut payload = Payload::new();
        let first_bytes = vec![0x16];

        // When
        payload.append_bytes(&first_bytes, 1).unwrap();

        // Then
        assert_eq!(payload.get_expected_length(), 23);
        assert_eq!(payload.get_remaining_to_read(), 22);
        assert_eq!(payload.get_bytes_received(), 1);
        assert!(!payload.is_complete());
    }

    #[test]
    fn test_read_first_two_bytes_at_once() {
        // Given
        let mut payload = Payload::new();
        let first_bytes = vec![0x10, 0x00];

        // When
        payload.append_bytes(&first_bytes, 2).unwrap();

        // Then
        assert_eq!(payload.get_expected_length(), 17);
        assert_eq!(payload.get_remaining_to_read(), 15);
        assert_eq!(payload.get_bytes_received(), 2);
        assert!(!payload.is_complete());
    }

    #[test]
    fn test_read_first_two_bytes_in_sequence() {
        // Given
        let mut payload = Payload::new();
        let first_bytes = vec![0x10];
        let next_bytes = vec![0x00];

        // When
        payload.append_bytes(&first_bytes, 1).unwrap();
        payload.append_bytes(&next_bytes, 1).unwrap();

        // Then
        assert_eq!(payload.get_expected_length(), 17);
        assert_eq!(payload.get_remaining_to_read(), 15);
        assert_eq!(payload.get_bytes_received(), 2);
        assert!(!payload.is_complete());
    }

    #[test]
    fn test_read_complete_payload_at_once() {
        // Given
        let mut payload = Payload::new();
        let first_bytes = vec![0x01, 0x00];
        let expected_data = vec![0x00];
        let expected_bytes = vec![0x01, 0x00];

        // When
        payload.append_bytes(&first_bytes, 2).unwrap();

        // Then
        assert_eq!(payload.get_expected_length(), 2);
        assert_eq!(payload.get_bytes_received(), 2);
        assert_eq!(payload.get_remaining_to_read(), 0);
        assert!(payload.is_complete());
        assert_eq!(payload.get_data(), expected_data);
        assert_eq!(payload.get_all_bytes(), expected_bytes);
        assert_eq!(payload.get_packet_size(), 1);
    }

    #[test]
    fn test_read_complete_payload_in_sequence() {
        // Given
        let mut payload = Payload::new();
        let first_bytes = vec![0x01];
        let next_bytes = vec![0x00];
        let expected_data = vec![0x00];

        // When
        payload.append_bytes(&first_bytes, 1).unwrap();
        payload.append_bytes(&next_bytes, 1).unwrap();

        // Then
        assert_eq!(payload.get_expected_length(), 2);
        assert_eq!(payload.get_remaining_to_read(), 0);
        assert_eq!(payload.get_bytes_received(), 2);
        assert!(payload.is_complete());
        assert_eq!(payload.get_data(), expected_data);
        assert_eq!(payload.get_packet_size(), 1);
    }

    #[test]
    fn test_read_large_size_slowly() {
        // Given
        let mut payload = Payload::new();
        let first_bytes = vec![0xdd];
        let next_bytes = vec![0xc7, 0x01];

        // When
        payload.append_bytes(&first_bytes, 1).unwrap();
        payload.append_bytes(&next_bytes, 2).unwrap();

        // Then
        assert_eq!(payload.get_expected_length(), 25565 + 3);
        assert_eq!(payload.get_remaining_to_read(), 25565);
        assert_eq!(payload.get_bytes_received(), 3);
        assert!(!payload.is_complete());
    }

    #[test]
    fn test_read_complete_handshake_payload() {
        // Given
        let mut payload = Payload::new();
        let localhost_handshake_packet = vec![
            0x10, // Packet length
            0x00, // Packet ID
            0xff, // Packet start
            0x05, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x63, 0xdd, 0x01,
        ];
        let expected_data = [
            0x00, // Packet ID
            0xff, // Packet start
            0x05, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x63, 0xdd, 0x01,
        ];

        // When
        for byte in localhost_handshake_packet {
            assert!(!payload.is_complete());
            payload.append_bytes(&[byte], 1).unwrap();
        }

        // Then
        assert_eq!(payload.get_expected_length(), 17);
        assert_eq!(payload.get_remaining_to_read(), 0);
        assert_eq!(payload.get_bytes_received(), 17);
        assert!(payload.is_complete());
        let payload_data = payload.get_data();
        for i in 0..expected_data.len() {
            assert_eq!(payload_data[i], expected_data[i]);
        }
        assert_eq!(payload.get_packet_size(), 16);
    }
}
