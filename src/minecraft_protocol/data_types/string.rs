use crate::minecraft_protocol::data_types::var_int::{read_var_int, CONTINUE_BIT};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Invalid string size error")]
pub(crate) struct InvalidStringSizeError;

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
