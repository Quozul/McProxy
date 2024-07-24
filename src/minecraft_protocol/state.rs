use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

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

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            State::Handshake => f.write_str("Handshake"),
            State::Status => f.write_str("Status"),
            State::Login => f.write_str("Login"),
            State::Transfer => f.write_str("Transfer"),
        }
    }
}
