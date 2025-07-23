use std::{env::var, net::UdpSocket, time::Duration};

use messages::client::ClientMessages;

pub fn get_ports() -> (u16, u16) {
    (
        var("SELF-PORT")
            .unwrap_or("4001".to_owned())
            .parse()
            .unwrap_or(4001),
        var("SERVER")
            .unwrap_or("4000".to_owned())
            .parse()
            .unwrap_or(4000),
    )
}

pub fn bind_socket(self_port: u16) -> UdpSocket {
    let socket = UdpSocket::bind(("127.0.0.1", self_port)).expect("could not bind port");

    socket
        .set_read_timeout(Some(Duration::new(5, 0)))
        .expect("whoops");

    socket
}

pub fn connect(socket: &UdpSocket, self_port: u16, server_port: u16) {
    let msg = ClientMessages::Connect { self_port };
    let msg_vec = Vec::from(&msg);

    if let Err(err) = socket.send_to(&msg_vec, ("127.0.0.1", server_port)) {
        log::error!("Could not connect to server at {server_port}\n{err:?}");
    }
}
