// THIS IS WHERE YOU SHOULD PUT YOUR CODE.

use messages::{client::ClientMessages, server::ServerMessages};

#[derive(Default)]
pub struct State {
    // put any persistent variables here
}

impl State {
    // THIS IS THE FUNCTION WHERE YOU CAN CONTROL YOUR TANK
    pub fn handle_message(&mut self, incoming: &ServerMessages) -> Option<ClientMessages> {
        match incoming {
            ServerMessages::MapChange {
                secret: _,
                walls: _,
            } => {
                // HANDLE MAP CHANGE -- clear current state?
                // No need for input, return None
                None
            }
            ServerMessages::State {
                secret,
                tanks: _,
                bullets: _,
            } => {
                // HANDLE GENERAL STATE CHANGE
                // put your logic here, potentially update your state

                // return your desired controls:
                Some(ClientMessages::control([0.0, 0.0], 0.0, true, *secret))
            }
            // In case we got dropped, try to reconnect.
            // No need to change this.
            ServerMessages::Disconnected => Some(ClientMessages::connect()),
        }
    }
}
