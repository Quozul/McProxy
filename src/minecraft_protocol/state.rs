use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub(crate) struct UnknownStateError;

impl fmt::Display for UnknownStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown state")
    }
}

impl Error for UnknownStateError {}

#[derive(Debug, PartialEq)]
pub(crate) enum State {
    Handshake,
    Status,
    Login,
    Transfer,
    //Configuration,
    //Play,
}
