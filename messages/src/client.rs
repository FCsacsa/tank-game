#[derive(Debug, PartialEq, Clone)]
pub enum ClientMessages {
    Connect {
        self_port: u16,
    },
    Control {
        // +1 byte
        self_port: u16,                       // 2 bytes
        secret: u128,                         // 16 bytes
        tracks_acceleration_target: [f32; 2], // 8 bytes
        turret_acceleration_target: f32,      // 4 bytes
        shoot: bool,                          // 1 byte
    },
}

impl ClientMessages {
    pub fn connect() -> Self {
        Self::Connect { self_port: 0 }
    }

    pub fn control(
        tracks_acceleration_target: [f32; 2],
        turret_acceleration_target: f32,
        shoot: bool,
    ) -> Self {
        Self::Control {
            self_port: 0,
            secret: 0,
            tracks_acceleration_target,
            turret_acceleration_target,
            shoot,
        }
    }

    pub fn set_port(&mut self, n_self_port: u16) {
        match self {
            ClientMessages::Connect { self_port }
            | ClientMessages::Control {
                self_port,
                secret: _,
                tracks_acceleration_target: _,
                turret_acceleration_target: _,
                shoot: _,
            } => *self_port = n_self_port,
        }
    }

    pub fn set_secret(&mut self, n_secret: u128) {
        match self {
            ClientMessages::Control { self_port: _, secret, tracks_acceleration_target: _, turret_acceleration_target: _, shoot: _ } => *secret = n_secret,
            _ => (),
        }
    }
}

impl From<&ClientMessages> for Vec<u8> {
    fn from(value: &ClientMessages) -> Self {
        match value {
            ClientMessages::Connect { self_port } => {
                vec![0x00, (self_port >> 8) as u8, *self_port as u8]
            }
            ClientMessages::Control {
                self_port,
                secret,
                tracks_acceleration_target,
                turret_acceleration_target,
                shoot,
            } => {
                let mut msg = vec![0x01];
                msg.append(&mut self_port.to_be_bytes().to_vec());
                msg.append(&mut secret.to_be_bytes().to_vec());
                msg.append(&mut tracks_acceleration_target[0].to_be_bytes().to_vec());
                msg.append(&mut tracks_acceleration_target[1].to_be_bytes().to_vec());
                msg.append(&mut turret_acceleration_target.to_be_bytes().to_vec());
                msg.push(if *shoot { 1 } else { 0 });
                msg
            }
        }
    }
}

impl TryFrom<&[u8]> for ClientMessages {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value[0] {
            0x00 => Ok(ClientMessages::Connect {
                self_port: ((value[1] as u16) << 8) + value[2] as u16,
            }),
            0x01 => Ok(ClientMessages::Control {
                self_port: u16::from_be_bytes(value[1..3].try_into().unwrap()),
                secret: u128::from_be_bytes(value[3..19].try_into().expect("u128 is 16 bytes")),
                tracks_acceleration_target: [
                    f32::from_be_bytes(value[19..23].try_into().expect("f32 is 4 bytes")),
                    f32::from_be_bytes(value[23..27].try_into().expect("f32 is 4 bytes")),
                ],
                turret_acceleration_target: f32::from_be_bytes(
                    value[27..31].try_into().expect("f32 is 4 bytes"),
                ),
                shoot: value[31] != 0,
            }),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::UNIX_EPOCH;

    use super::*;

    #[test]
    fn test_client() {
        let cm = ClientMessages::Connect {
            self_port: (UNIX_EPOCH.elapsed().unwrap().as_secs() % (u16::MAX as u64)) as u16,
        };
        println!("{:?} -> {:?}", cm, Vec::from(&cm));
        assert_eq!(Vec::from(&cm).len(), 3);
        assert_eq!(ClientMessages::try_from(&Vec::from(&cm)[..]).unwrap(), cm);
        let cm = ClientMessages::Control {
            self_port: rand::random(),
            secret: rand::random(),
            tracks_acceleration_target: [rand::random(), rand::random()],
            turret_acceleration_target: rand::random(),
            shoot: true,
        };
        println!("{:?} -> {:?}", cm, Vec::from(&cm));
        assert_eq!(Vec::from(&cm).len(), 32);
        assert_eq!(ClientMessages::try_from(&Vec::from(&cm)[..]).unwrap(), cm);
    }
}
