use messages::{client::ClientMessages, server::ServerMessages};

#[derive(Default)]
pub struct State {
    // put any persistent variables here
}

impl State {
    pub fn handle_message(&mut self, incoming: &ServerMessages) -> Option<ClientMessages> {
        match incoming {
            ServerMessages::MapChange {
                secret: _,
                walls: _,
            } => None,
            ServerMessages::State {
                secret,
                tanks: _,
                bullets: _,
            } => Some(ClientMessages::control([0.0, 0.0], 0.0, true, *secret)),
            ServerMessages::Disconnected => Some(ClientMessages::connect()),
        }
    }
}
