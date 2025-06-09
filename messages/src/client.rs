#[derive(Debug, PartialEq, Clone)]
pub enum ClientMessages {
    Connect {
        self_port: u16,
    },
    Control {
        secret: u128,
        tracks_acceleration_target: [f32; 2],
        turret_acceleration_target: f32,
        shoot: bool,
    },
}

impl From<&ClientMessages> for Vec<u8> {
    fn from(value: &ClientMessages) -> Self {
        match value {
            ClientMessages::Connect { self_port } => {
                vec![0x00, (self_port >> 8) as u8, *self_port as u8]
            }
            ClientMessages::Control {
                secret,
                tracks_acceleration_target,
                turret_acceleration_target,
                shoot,
            } => {
                let mut msg = vec![0x01];
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
                secret: u128::from_be_bytes(value[1..17].try_into().expect("u128 is 16 bytes")),
                tracks_acceleration_target: [
                    f32::from_be_bytes(value[17..21].try_into().expect("f32 is 4 bytes")),
                    f32::from_be_bytes(value[21..25].try_into().expect("f32 is 4 bytes")),
                ],
                turret_acceleration_target: f32::from_be_bytes(
                    value[25..29].try_into().expect("f32 is 4 bytes"),
                ),
                shoot: value[29] != 0,
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
        assert_eq!(
            ClientMessages::try_from(&Vec::from(&cm)[..]).unwrap(),
            cm
        );
        let cm = ClientMessages::Control {
            secret: UNIX_EPOCH.elapsed().unwrap().as_millis(),
            tracks_acceleration_target: [
                UNIX_EPOCH.elapsed().unwrap().as_secs_f32(),
                UNIX_EPOCH.elapsed().unwrap().as_secs_f32() - 100.0,
            ],
            turret_acceleration_target: UNIX_EPOCH.elapsed().unwrap().as_secs_f32() - 300.0,
            shoot: true,
        };
        println!("{:?} -> {:?}", cm, Vec::from(&cm));
        assert_eq!(Vec::from(&cm).len(), 30);
        assert_eq!(
            ClientMessages::try_from(&Vec::from(&cm)[..]).unwrap(),
            cm
        );
    }
}
