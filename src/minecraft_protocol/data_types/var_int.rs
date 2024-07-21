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

pub(crate) fn read_var_int(bytes: &[u8], index: &mut usize) -> Result<i32, Box<dyn Error>> {
    let mut value = 0;
    let mut position = 0;

    while position < 32 {
        let current_byte = bytes[*index];
        value |= ((current_byte & SEGMENT_BITS) as i32) << position;

        if (current_byte & CONTINUE_BIT) == 0 {
            *index += 1;
            break;
        }

        position += 7;
        *index += 1;
    }

    if position >= 32 {
        return Err(Box::new(VarIntTooBigError));
    }

    Ok(value)
}
