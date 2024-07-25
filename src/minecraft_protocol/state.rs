use std::fmt;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("unknown state")]
pub(crate) struct UnknownStateError;

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
