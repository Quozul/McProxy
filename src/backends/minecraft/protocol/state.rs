use std::fmt;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("unknown state {0}")]
pub(crate) struct UnknownStateError(i32);

#[derive(Debug, PartialEq)]
pub(crate) enum State {
    Handshake,
    Status,
    Login,
    Transfer,
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

impl State {
    pub(crate) fn parse(state: i32) -> Result<Self, UnknownStateError> {
        match state {
            0 => Ok(State::Handshake),
            1 => Ok(State::Status),
            2 => Ok(State::Login),
            3 => Ok(State::Transfer),
            _ => Err(UnknownStateError(state)),
        }
    }
}
