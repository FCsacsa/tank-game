use std::{env, net::UdpSocket, time::Duration};

use messages::{
    client::ClientMessages,
    server::{Bullet, ServerMessages, Tank, Wall},
};

type FnMapChange = dyn Fn(&[Wall]) -> Option<ClientMessages>;
type FnStateChange = dyn Fn(&[Tank], &[Bullet]) -> Option<ClientMessages>;

type ClientError = String;

pub struct TankClient<'a> {
    socket: UdpSocket,

    server: u16,
    self_port: u16,

    pub handle_map_change: &'a FnMapChange,
    pub handle_state_change: &'a FnStateChange,
}

impl<'a> TankClient<'a> {
    pub fn new(handle_map_change: &'a FnMapChange, handle_state_change: &'a FnStateChange) -> Self {
        let self_port = env::var("SELF-PORT")
            .unwrap_or("4001".to_owned())
            .parse()
            .unwrap_or(4001);
        let server = env::var("SERVER")
            .unwrap_or("4000".to_owned())
            .parse()
            .unwrap_or(4000);

        let socket = UdpSocket::bind(("127.0.0.1", self_port)).expect("could not bind port");
        socket
            .set_read_timeout(Some(Duration::new(5, 0)))
            .expect("whoops");

        Self {
            socket,
            server,
            self_port,
            handle_map_change,
            handle_state_change,
        }
    }

    fn connect(&self) -> Result<(), ClientError> {
        let msg = ClientMessages::Connect {
            self_port: self.self_port,
        };
        let msg_vec = Vec::from(&msg);

        self.socket
            .send_to(&msg_vec, ("127.0.0.1", self.server))
            .map(|_| ())
            .map_err(|err| format!("Could not connect to server at {}\n{err:?}", self.server))
    }

    pub fn run(&self) -> Result<(), ClientError> {
        loop {
            self.connect()?;

            let mut buf = [0; 4885]; // at minimum 4883

            while self.socket.recv(&mut buf).is_ok() {
                match ServerMessages::try_from(&buf[..]) {
                    Ok(ServerMessages::MapChange { secret, walls }) => {
                        if let Some(mut msg) = (self.handle_map_change)(&walls) {
                            msg.set_port(self.self_port);
                            msg.set_secret(secret);
                            self.socket
                                .send_to(&Vec::from(&msg), ("127.0.0.1", self.server))
                                .map_err(|err| {
                                    format!("Could not send the message to the server:\n{err}")
                                })?;
                        }
                    }
                    Ok(ServerMessages::State {
                        secret,
                        tanks,
                        bullets,
                    }) => if let Some(mut msg) = (self.handle_state_change)(&tanks, &bullets) {
                            msg.set_port(self.self_port);
                            msg.set_secret(secret);
                            self.socket
                                .send_to(&Vec::from(&msg), ("127.0.0.1", self.server))
                                .map_err(|err| {
                                    format!("Could not send the message to the server:\n{err}")
                                })?;
                    },
                    Ok(ServerMessages::Disconnected) => self.connect()?,
                    Err(err) => Err(format!("received an ill-formatted message:\n{err:?}"))?,
                }
            }
        }
    }
}
