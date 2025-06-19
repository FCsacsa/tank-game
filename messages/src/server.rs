#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wall {
    pub origin: [f32; 2],
    pub direction_length: [f32; 2],
}

impl From<&Wall> for Vec<u8> {
    fn from(value: &Wall) -> Self {
        let mut vec = value.origin.to_vec();
        vec.append(&mut value.direction_length.to_vec());
        vec.iter_mut()
            .map(|f| f.to_be_bytes())
            .collect::<Vec<_>>()
            .concat()
    }
}

impl From<&[u8; 16]> for Wall {
    fn from(value: &[u8; 16]) -> Self {
        Wall {
            origin: [
                f32::from_be_bytes(value[0..4].try_into().unwrap()),
                f32::from_be_bytes(value[4..8].try_into().unwrap()),
            ],
            direction_length: [
                f32::from_be_bytes(value[8..12].try_into().unwrap()),
                f32::from_be_bytes(value[12..16].try_into().unwrap()),
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tank {
    pub position: [f32; 2],
    pub tank_direction: [f32; 2],
    pub turret_direction: [f32; 2],
}

impl From<&Tank> for Vec<u8> {
    fn from(value: &Tank) -> Self {
        let mut vec = value.position.to_vec();
        vec.append(&mut value.tank_direction.to_vec());
        vec.append(&mut value.turret_direction.to_vec());
        vec.iter_mut()
            .map(|f| f.to_be_bytes())
            .collect::<Vec<_>>()
            .concat()
    }
}

impl From<&[u8; 24]> for Tank {
    fn from(value: &[u8; 24]) -> Self {
        Tank {
            position: [
                f32::from_be_bytes(value[0..4].try_into().unwrap()),
                f32::from_be_bytes(value[4..8].try_into().unwrap()),
            ],
            tank_direction: [
                f32::from_be_bytes(value[8..12].try_into().unwrap()),
                f32::from_be_bytes(value[12..16].try_into().unwrap()),
            ],
            turret_direction: [
                f32::from_be_bytes(value[16..20].try_into().unwrap()),
                f32::from_be_bytes(value[20..24].try_into().unwrap()),
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bullet {
    pub position: [f32; 2],
    pub direction: [f32; 2],
}

impl From<&Bullet> for Vec<u8> {
    fn from(value: &Bullet) -> Self {
        let mut vec = value.position.to_vec();
        vec.append(&mut value.direction.to_vec());
        vec.iter_mut()
            .map(|f| f.to_be_bytes())
            .collect::<Vec<_>>()
            .concat()
    }
}

impl From<&[u8; 16]> for Bullet {
    fn from(value: &[u8; 16]) -> Self {
        Bullet {
            position: [
                f32::from_be_bytes(value[0..4].try_into().unwrap()),
                f32::from_be_bytes(value[4..8].try_into().unwrap()),
            ],
            direction: [
                f32::from_be_bytes(value[8..12].try_into().unwrap()),
                f32::from_be_bytes(value[12..16].try_into().unwrap()),
            ],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerMessages {
    MapChange {
        // + 1 byte
        secret: u128,     // 16 bytes
        walls: Vec<Wall>, // 256 * 16 bytes
    },
    State {
        // + 1 byte
        secret: u128,         // 16 bytes
        tanks: Vec<Tank>,     // 32 * 24 + 1 bytes
        bullets: Vec<Bullet>, // 256 * 16 + 1 bytes
    },
    Disconnected,
}
// total of up to: 4883 bytes

impl ServerMessages {
    pub fn to_vec(self) -> Vec<u8> {
        Vec::from(&self)
    }
}

impl From<&ServerMessages> for Vec<u8> {
    fn from(value: &ServerMessages) -> Self {
        match value {
            ServerMessages::MapChange { secret, walls } => {
                let mut vec = vec![0x00];
                vec.append(&mut secret.to_be_bytes().to_vec());
                vec.push(walls.len() as u8);
                walls.iter().for_each(|w| vec.append(&mut Vec::from(w)));
                vec
            }
            ServerMessages::State {
                secret,
                tanks,
                bullets,
            } => {
                let mut vec = vec![0x01];
                vec.append(&mut secret.to_be_bytes().to_vec());
                vec.push(tanks.len() as u8);
                tanks.iter().for_each(|t| vec.append(&mut Vec::from(t)));
                vec.push(bullets.len() as u8);
                bullets.iter().for_each(|b| vec.append(&mut Vec::from(b)));
                vec
            }
            ServerMessages::Disconnected => vec![0x02],
        }
    }
}

impl TryFrom<&[u8]> for ServerMessages {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value[0] {
            0x00 => {
                let secret = u128::from_be_bytes(value[1..17].try_into().unwrap());
                let wall_count = value[17] as usize;
                let mut walls = vec![];
                for i in 0..wall_count {
                    walls.push(Wall::from(
                        &value[18 + i * 16..34 + i * 16].try_into().unwrap(),
                    ))
                }
                Ok(Self::MapChange { secret, walls })
            }
            0x01 => {
                let secret = u128::from_be_bytes(value[1..17].try_into().unwrap());
                let tank_count = value[17] as usize;
                let mut tanks = vec![];
                for i in 0..tank_count {
                    tanks.push(Tank::from(
                        &value[18 + i * 24..42 + i * 24].try_into().unwrap(),
                    ))
                }
                let start = 18 + tank_count * 24;
                let bullet_count = value[start] as usize;
                let start = start + 1;
                let mut bullets = vec![];
                for i in 0..bullet_count {
                    bullets.push(Bullet::from(
                        &value[start + i * 16..start + (i + 1) * 16]
                            .try_into()
                            .unwrap(),
                    ))
                }
                Ok(Self::State {
                    secret,
                    tanks,
                    bullets,
                })
            }
            0x02 => Ok(Self::Disconnected),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walls() {
        let wall_count = rand::random_range(0..32);
        let mut walls = vec![];
        for _ in 0..wall_count {
            walls.push(Wall {
                origin: [rand::random(), rand::random()],
                direction_length: [rand::random(), rand::random()],
            })
        }
        let map = ServerMessages::MapChange {
            secret: rand::random(),
            walls,
        };
        assert_eq!(Vec::from(&map).len(), 18 + wall_count * 16);
        assert_eq!(ServerMessages::try_from(&Vec::from(&map)[..]).unwrap(), map);
    }

    #[test]
    fn test_status() {
        let tank_count = rand::random_range(0..8);
        let bullet_count = rand::random_range(0..32);
        let state = ServerMessages::State {
            secret: rand::random(),
            tanks: (0..tank_count)
                .map(|_| Tank {
                    position: [rand::random(), rand::random()],
                    tank_direction: [rand::random(), rand::random()],
                    turret_direction: [rand::random(), rand::random()],
                })
                .collect(),
            bullets: (0..bullet_count)
                .map(|_| Bullet {
                    position: [rand::random(), rand::random()],
                    direction: [rand::random(), rand::random()],
                })
                .collect(),
        };

        assert_eq!(
            Vec::from(&state).len(),
            19 + tank_count * 24 + bullet_count * 16
        );
        match ServerMessages::try_from(&Vec::from(&state)[..]).unwrap() {
            ServerMessages::State { tanks, bullets, .. } => {
                assert_eq!(tanks.len(), tank_count);
                assert_eq!(bullets.len(), bullet_count);
            }
            _ => assert!(false, "Something is very wrong"),
        }
        assert_eq!(
            ServerMessages::try_from(&Vec::from(&state)[..]).unwrap(),
            state
        );
    }

    #[test]
    fn test_disconnect() {
        assert_eq!(
            ServerMessages::try_from(&Vec::from(&ServerMessages::Disconnected)[..]).unwrap(),
            ServerMessages::Disconnected
        );
    }
}
