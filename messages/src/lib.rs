use std::fmt::Display;

pub enum ClientMessages {
    ConnectMessage {
        port: u16,
    },
    ControlMessage {
        target_acceleration: [f32; 2], // 2 x 4 bytes
        turret_acceleration: f32,      // 4 bytes
        shoot: bool,                   // 1 byte
    },
}

impl Display for ClientMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientMessages::ConnectMessage { port } => {
                write!(f, "incoming connection from {}", port)
            }
            ClientMessages::ControlMessage {
                target_acceleration,
                turret_acceleration,
                shoot,
            } => write!(
                f,
                "targets: acceleration - [{}, {}]; turret - {}{}",
                target_acceleration[0],
                target_acceleration[1],
                *turret_acceleration,
                if *shoot { ", shoot" } else { "" }
            ),
        }
    }
}

impl From<ClientMessages> for Vec<u8> {
    fn from(value: ClientMessages) -> Self {
        match value {
            ClientMessages::ConnectMessage {port} => vec![[0, 0], port.to_be_bytes()].concat(),
            ClientMessages::ControlMessage {
                target_acceleration,
                turret_acceleration,
                shoot,
            } => {
                let acceleration_x = target_acceleration[0].to_be_bytes();
                let acceleration_y = target_acceleration[1].to_be_bytes();
                let acceleration_turret = turret_acceleration.to_be_bytes();
                let shoot = shoot as u8;
                [
                    [1, shoot, 0, 0],
                    acceleration_x,
                    acceleration_y,
                    acceleration_turret,
                ]
                .concat()
            }
        }
    }
}

impl From<[u8; 16]> for ClientMessages {
    fn from(value: [u8; 16]) -> Self {
        match value[0] {
            0 => {
                let port = u16::from_be_bytes(value[2..6].try_into().expect("4 bytes are 4 bytes"));
                Self::ConnectMessage {port}
            },
            1 => {
                let acceleration_x =
                    f32::from_be_bytes(value[4..8].try_into().expect("4 bytes is 4 bytes"));
                let acceleration_y =
                    f32::from_be_bytes(value[8..12].try_into().expect("4 bytes is 4 bytes"));
                let acceleration_turret =
                    f32::from_be_bytes(value[12..16].try_into().expect("4 bytes is 4 bytes"));
                let shoot = value[1] != 0;
                Self::ControlMessage {
                    target_acceleration: [acceleration_x, acceleration_y],
                    turret_acceleration: acceleration_turret,
                    shoot,
                }
            }
            _ => todo!("incorrect message"),
        }
    }
}

pub struct TankState {
    position: [f32; 2], // 2 * 4 bytes
    facing: [f32; 2],   // 2 * 4 bytes
    turret: [f32; 2],   // 2 * 4 bytes
}

impl Display for TankState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pos: [{}, {}], facing: [{}, {}], turret: [{}, {}]",
            self.position[0],
            self.position[1],
            self.facing[0],
            self.facing[1],
            self.turret[0],
            self.turret[1]
        )
    }
}

pub enum ServerMessages {
    MapChange,
    TankStates {
        client_id: u8, // we expect less than 16 clients  | 1 byte + 3 for padding and msg type
        tanks: Vec<TankState>, // position and state of all tanks | 16 * 24 bytes
        bullets: Vec<[f32; 2]>, // position of all active bullets  | 128 * 2 * 4 bytes
    },
}

impl Display for ServerMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerMessages::MapChange => todo!(),
            ServerMessages::TankStates {
                client_id,
                tanks,
                bullets,
            } => {
                write!(f, "game state:\n\tclient-id: {}\n\ttanks:\n", client_id)?;
                for (i, tank) in tanks.iter().enumerate() {
                    write!(f, "\t\t{i}: {tank}\n")?;
                }
                write!(f, "\tbullets:")?;
                for bullet in bullets {
                    write!(f, "\n\t\t[{}, {}]", bullet[0], bullet[1])?;
                }
                write!(f, "")
            }
        }
    }
}

impl From<ServerMessages> for Vec<u8> {
    fn from(value: ServerMessages) -> Self {
        match value {
            ServerMessages::MapChange => todo!(),
            ServerMessages::TankStates {
                client_id,
                tanks,
                bullets,
            } => {
                let mut collect: Vec<[u8; 4]> = Vec::new();
                collect.push([1, client_id, tanks.len() as u8, bullets.len() as u8]);
                for TankState {
                    position,
                    facing,
                    turret,
                } in tanks
                {
                    collect.push(position[0].to_be_bytes());
                    collect.push(position[1].to_be_bytes());
                    collect.push(facing[0].to_be_bytes());
                    collect.push(facing[1].to_be_bytes());
                    collect.push(turret[0].to_be_bytes());
                    collect.push(turret[1].to_be_bytes());
                }
                for bullet in bullets {
                    collect.push(bullet[0].to_be_bytes());
                    collect.push(bullet[1].to_be_bytes());
                }

                collect.concat()
            }
        }
    }
}

impl TryFrom<[u8; 1412]> for ServerMessages {
    type Error = ();

    fn try_from(value: [u8; 1412]) -> Result<Self, Self::Error> {
        match value[0] {
            0 => todo!(),
            1 => {
                let client_id = value[1];
                let n_tanks = value[2];
                let n_bullets = value[3];

                let mut tanks = Vec::new();
                for i in 0..n_tanks as usize {
                    let start = 4 + 24 * i;
                    tanks.push(TankState {
                        position: [
                            f32::from_be_bytes(
                                value[start..start + 4]
                                    .try_into()
                                    .expect("4 bytes is 4 bytes"),
                            ),
                            f32::from_be_bytes(
                                value[start + 4..start + 8]
                                    .try_into()
                                    .expect("4 bytes is 4 bytes"),
                            ),
                        ],
                        facing: [
                            f32::from_be_bytes(
                                value[start + 8..start + 12]
                                    .try_into()
                                    .expect("4 bytes is 4 bytes"),
                            ),
                            f32::from_be_bytes(
                                value[start + 12..start + 16]
                                    .try_into()
                                    .expect("4 bytes is 4 bytes"),
                            ),
                        ],
                        turret: [
                            f32::from_be_bytes(
                                value[start + 16..start + 20]
                                    .try_into()
                                    .expect("4 bytes is 4 bytes"),
                            ),
                            f32::from_be_bytes(
                                value[start + 20..start + 24]
                                    .try_into()
                                    .expect("4 bytes is 4 bytes"),
                            ),
                        ],
                    })
                }

                let mut bullets = Vec::new();
                for i in 0..n_bullets as usize {
                    let start = 4 + n_tanks as usize * 24 + i * 8;
                    bullets.push([
                        f32::from_be_bytes(
                            value[start..start + 4]
                                .try_into()
                                .expect("4 bytes is 4 bytes"),
                        ),
                        f32::from_be_bytes(
                            value[start + 4..start + 8]
                                .try_into()
                                .expect("4 bytes is 4 bytes"),
                        ),
                    ]);
                }

                Ok(Self::TankStates {
                    client_id,
                    tanks,
                    bullets,
                })
            }
            _ => Err(()),
        }
    }
}
