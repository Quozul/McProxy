use std::error::Error;
use std::fmt;

use crate::minecraft_protocol::data_types::var_int::{read_var_int, CONTINUE_BIT};

#[derive(Debug)]
pub(crate) struct StringTooBigError;

impl fmt::Display for StringTooBigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "String is too big")
    }
}

impl Error for StringTooBigError {}

pub(crate) fn read_string(bytes: &[u8], index: &mut usize) -> Result<String, Box<dyn Error>> {
    let length = read_var_int(bytes, index)?;

    if length > 255 {
        return Err(Box::new(StringTooBigError));
    }

    while (bytes[*index] & CONTINUE_BIT) != 0 {
        *index += 1;
    }

    let result = std::str::from_utf8(&bytes[*index..*index + length as usize])
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    *index += length as usize;

    Ok(result.to_string())
}
