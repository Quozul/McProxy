use std::error::Error;
use std::fmt;

use crate::minecraft_protocol::data_types::var_int::{read_var_int, CONTINUE_BIT};

#[derive(Debug)]
pub(crate) struct InvalidStringSizeError;

impl fmt::Display for InvalidStringSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid string size error")
    }
}

impl Error for InvalidStringSizeError {}

pub(crate) fn read_string(bytes: &[u8], index: &mut usize) -> Result<String, Box<dyn Error>> {
    let length = read_var_int(bytes, index)? as usize;

    if length > 255 {
        return Err(Box::new(InvalidStringSizeError));
    }

    while (bytes[*index] & CONTINUE_BIT) != 0 {
        *index += 1;
    }

    let result = std::str::from_utf8(&bytes[*index..*index + length])?;

    *index += length;

    Ok(result.to_string())
}
