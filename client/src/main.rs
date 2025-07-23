use messages::server::ServerMessages;

use client::*;

use crate::team_code::State;

mod team_code;

fn main() {
    let (self_port, server_port) = get_ports();
    let socket = bind_socket(self_port);

    loop {
        connect(&socket, self_port, server_port);

        let mut buf = [0; 4885]; // at minimum 4883
        let mut state = State::default();

        while let Ok(_) = socket.recv(&mut buf) {
            let parsed =
                ServerMessages::try_from(&buf[..]).expect("should not have ill formatted messages");

            if let Some(mut response) = state.handle_message(&parsed) {
                response.set_port(self_port);
                if let Err(err) = socket.send_to(&Vec::from(&response), ("127.0.0.1", server_port)) {
                    log::error!("Could not send input to server at {server_port}:\n{err:?}");
                }
            }
            
        }
    }
}
