use std::error::Error;
use std::string::String;

use crate::minecraft_protocol::data_types::string::read_string;
use crate::minecraft_protocol::data_types::unsigned_short::read_unsigned_short;
use crate::minecraft_protocol::data_types::var_int::read_var_int;

#[derive(Debug)]
pub(crate) struct McHandshake {
    pub(crate) protocol: i32,
    pub(crate) hostname: String,
    pub(crate) port: u16,
    pub(crate) next_state: i32,
}

pub(crate) fn handle_handshake(
    bytes: &[u8],
    index: &mut usize,
) -> Result<McHandshake, Box<dyn Error>> {
    let protocol = read_var_int(bytes, index)?;
    let hostname = read_string(bytes, index)?;
    let port = read_unsigned_short(bytes, index);
    let next_state = read_var_int(bytes, index)?;

    Ok(McHandshake {
        protocol,
        hostname,
        port,
        next_state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_handshake() {
        let localhost_handshake_packet = vec![
            0xff, 0x05, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x63, 0xdd,
            0x01,
        ];

        let mut index = 0;
        let handshake = handle_handshake(&localhost_handshake_packet, &mut index).unwrap();

        assert_eq!(handshake.protocol, 767);
        assert_eq!(handshake.hostname, "localhost");
        assert_eq!(handshake.port, 25565);
        assert_eq!(handshake.next_state, 1);
    }
}
