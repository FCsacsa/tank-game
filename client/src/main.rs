use client_lib_rs::TankClient;
use messages::{
    client::ClientMessages,
    server::{Bullet, Tank, Wall},
};

fn handle_map_change(walls: &[Wall]) -> Option<ClientMessages> {
    todo!(
        "Handle the map changing to have the following walls:\n{:?}",
        walls
    )
}
fn handle_state_change(tanks: &[Tank], bullets: &[Bullet]) -> Option<ClientMessages> {
    todo!("Handle state change:\n{:?}\n{:?}\n", tanks, bullets)
}

fn main() {
    loop {
        if let Err(err) = TankClient::new(&handle_map_change, &handle_state_change).run() {
            log::error!("Something failed when running:\n{err}");
        }
    }
}
