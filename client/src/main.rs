use std::env::var;
use std::net::UdpSocket;

use messages::{ClientMessages, ServerMessages};

fn main() {
    let self_port = var("SELF-PORT")
        .unwrap_or("4001".to_owned())
        .parse()
        .unwrap_or(4001);
    let server_port = var("SERVER")
        .unwrap_or("4000".to_owned())
        .parse()
        .unwrap_or(4000);
    let socket = UdpSocket::bind(("127.0.0.1", self_port)).expect("could not bind port");

    let msg = ClientMessages::ConnectMessage { port: self_port };
    let msg_vec : Vec<_> = msg.into();

    socket
        .send_to(&msg_vec, ("127.0.0.1", server_port))
        .expect("could not send");

    let mut buf = [0; 1412];

    while let Ok(msg) = socket.recv(&mut buf) {
        let parsed : ServerMessages = buf.try_into().expect("should not have ill formatted messages");
        println!("{parsed}");
    }
}
