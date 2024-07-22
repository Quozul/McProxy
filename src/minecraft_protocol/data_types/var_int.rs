use std::error::Error;
use std::fmt;

pub(crate) const SEGMENT_BITS: u8 = 0x7F;
pub(crate) const CONTINUE_BIT: u8 = 0x80;

#[derive(Debug)]
pub(crate) struct VarIntTooBigError;

impl fmt::Display for VarIntTooBigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VarInt is too big")
    }
}

impl Error for VarIntTooBigError {}

pub(crate) fn read_var_int(bytes: &[u8], index: &mut usize) -> Result<i32, VarIntTooBigError> {
    let mut value = 0;
    let mut position = 0;

    while position < 32 {
        let current_byte = bytes[*index];
        value |= ((current_byte & SEGMENT_BITS) as i32) << position;

        *index += 1;
        if (current_byte & CONTINUE_BIT) == 0 {
            break;
        }

        position += 7;
    }

    if position >= 32 {
        return Err(VarIntTooBigError);
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_var_int() {
        let test_cases = vec![
            (vec![0x00], 0),
            (vec![0x01], 1),
            (vec![0x02], 2),
            (vec![0x7f], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xff, 0x01], 255),
            (vec![0xdd, 0xc7, 0x01], 25565),
            (vec![0xff, 0xff, 0x7f], 2097151),
            (vec![0xff, 0xff, 0xff, 0xff, 0x07], 2147483647),
            (vec![0xff, 0xff, 0xff, 0xff, 0x0f], -1),
            (vec![0x80, 0x80, 0x80, 0x80, 0x08], -2147483648),
        ];

        for (bytes, expected) in test_cases {
            let mut index = 0;
            let result = read_var_int(&bytes, &mut index);
            assert_eq!(result.unwrap(), expected);
        }
    }
}
