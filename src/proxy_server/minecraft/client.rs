use tokio::net::TcpStream;

use crate::minecraft_protocol::state::State;

pub(crate) struct Client {
    pub(crate) socket: TcpStream,
    pub(crate) state: State,
}

impl Client {
    pub(crate) fn new(socket: TcpStream) -> Client {
        Client {
            socket,
            state: State::Handshake,
        }
    }

    pub(crate) fn update_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    pub(crate) fn is_handshaking(&self) -> bool {
        self.state == State::Handshake
    }
}
