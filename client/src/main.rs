use std::env::var;
use std::net::UdpSocket;

use messages::{client::ClientMessages, server::ServerMessages};

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

    let msg = ClientMessages::Connect { self_port };
    let msg_vec = Vec::from(&msg);

    socket
        .send_to(&msg_vec, ("127.0.0.1", server_port))
        .expect("could not send");

    let mut buf = [0; 4885]; // at minimum 4883

    while let Ok(msg) = socket.recv(&mut buf) {
        let parsed =
            ServerMessages::try_from(&buf[..]).expect("should not have ill formatted messages");
        println!("{parsed:?}");
        match parsed {
            ServerMessages::MapChange { secret, walls } => {
                let msg_vec = Vec::from(&ClientMessages::Control {
                    secret,
                    tracks_acceleration_target: walls[0].origin,
                    turret_acceleration_target: walls[0].direction_length[1],
                    shoot: secret % 2 == 1,
                });
                socket
                    .send_to(&msg_vec, ("127.0.0.1", server_port))
                    .unwrap();
            }
            ServerMessages::State {
                secret,
                tanks,
                bullets,
            } => {
                socket
                    .send_to(
                        &Vec::from(&ClientMessages::Control {
                            secret,
                            tracks_acceleration_target: tanks[0].position,
                            turret_acceleration_target: tanks[0].tank_direction[0],
                            shoot: true,
                        }),
                        ("127.0.0.1", server_port),
                    )
                    .unwrap();
            }
            ServerMessages::Disconnected => {
                socket
                    .send_to(
                        &Vec::from(&ClientMessages::Connect { self_port }),
                        ("127.0.0.1", server_port),
                    )
                    .unwrap();
            }
        }
    }
}
