use bevy::{
    app::{App, Startup, Update},
    asset::AssetServer,
    diagnostic::FrameTimeDiagnosticsPlugin,
    math::Vec2,
    prelude::{Camera2d, Commands, Msaa, OrthographicProjection, Res},
    utils::default,
    DefaultPlugins,
};

mod tank;
use iyes_perf_ui::{
    prelude::{PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot},
    PerfUiPlugin,
};
use tank::{move_tanks, update_turrets, Tank};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_tanks, update_turrets))
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

    Tank::setup(
        "tank_body.png",
        "tank_turret.png",
        Vec2::new(-50.0, 0.0),
        &mut commands,
        &asset_server,
    );
    Tank::setup(
        "tank_body.png",
        "tank_turret.png",
        Vec2::new(50.0, 0.0),
        &mut commands,
        &asset_server,
    );

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
