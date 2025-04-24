use std::path::Path;

use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::AssetServer,
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::schedule::IntoSystemConfigs,
    math::Vec2,
    prelude::{Camera2d, Commands, Msaa, OrthographicProjection, Res},
    utils::default,
};
use iyes_perf_ui::{
    PerfUiPlugin,
    prelude::{PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot},
};

mod map;
mod util;
use map::{Map, wall_collision};
mod tank;
use tank::{Tank, move_tanks, update_turrets};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            ((move_tanks, wall_collision).chain(), update_turrets),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scale: 0.8,
            ..OrthographicProjection::default_2d()
        },
        Msaa::Off,
    ));

    Map::load_map_from_path(Path::new("assets/map.jsonc")).unwrap().setup(&mut commands, &asset_server);

    Tank::setup(
        "tank_body.png",
        "tank_turret.png",
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 100.0),
        &mut commands,
        &asset_server,
    );

    Tank::setup(
        "tank_body.png",
        "tank_turret.png",
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 100.0),
        &mut commands,
        &asset_server,
    );

    Tank::setup(
        "tank_body.png",
        "tank_turret.png",
        Vec2::new(-113.0, 0.0),
        Vec2::new(100.0, 100.0),
        &mut commands,
        &asset_server,
    );

    // Tank::setup(
    //     "tank_body.png",
    //     "tank_turret.png",
    //     Vec2::new(50.0, 0.0),
    //     Vec2::new(300.0, 500.0),
    //     &mut commands,
    //     &asset_server,
    // );

    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            values_col_width: 32.0,
            ..default()
        },
        PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
}
