use client_lib_rs::TankClient;
use messages::{
    client::ClientMessages,
    server::{Bullet, Tank, Wall},
};
use rand::{random, random_bool, random_range};

fn handle_map_change(walls: &[Wall]) -> Option<ClientMessages> {
    todo!(
        "Handle the map changing to have the following walls:\n{:?}",
        walls
    )
}
fn handle_state_change(_tanks: &[Tank], _bullets: &[Bullet]) -> Option<ClientMessages> {
    Some(ClientMessages::control(
        [
            if random_bool(0.6) { 1000.0 } else { -1000.0 },
            if random_bool(0.6) { 1000.0 } else { -1000.0 },
        ],
        if random_bool(0.5) { 10.0 } else { -10.0 },
        false,
        // random_bool(0.6),
    ))
}

fn main() {
    loop {
        if let Err(err) = TankClient::new(&handle_map_change, &handle_state_change).run() {
            log::error!("Something failed when running:\n{err}");
        }
    }
}
