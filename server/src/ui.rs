use bevy::{
    color::Color,
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::Children,
        query::With,
        system::{Commands, Query},
    },
    log,
    ui::{
        AlignItems, BackgroundColor, BorderRadius, BoxSizing, Display, GridAutoFlow, JustifyItems,
        Node, PositionType, RepeatedGridTrack, UiRect, Val, widget::Text,
    },
};

use crate::entities::Player;

#[derive(Component)]
#[require(Node)]
pub struct Leaderboard;

pub fn setup_leaderboard(mut commands: Commands) {
    commands.spawn((
        Leaderboard,
        Node {
            display: Display::Grid,
            box_sizing: BoxSizing::ContentBox,
            position_type: PositionType::Absolute,
            left: Val::Px(4.0),
            top: Val::Px(4.0),
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            padding: UiRect {
                left: Val::Px(4.0),
                right: Val::Px(4.0),
                top: Val::Px(4.0),
                bottom: Val::Px(4.0),
            },
            row_gap: Val::Px(2.0),
            column_gap: Val::Px(4.0),
            grid_auto_flow: GridAutoFlow::Row,
            grid_template_columns: vec![RepeatedGridTrack::auto(2)],
            ..Default::default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        BorderRadius::new(Val::Px(8.0), Val::Px(8.0), Val::Px(8.0), Val::Px(8.0)),
    ));
}

pub fn show_leaderboard(
    mut commands: Commands,
    leaderboard: Query<Entity, With<Leaderboard>>,
    players: Query<&Player>,
) {
    if players.is_empty() {
        return;
    }
    let mut players = players
        .iter()
        .map(|player| (player.port, player.get_deaths()))
        .collect::<Vec<_>>();
    players.sort_by(|a, b| a.1.cmp(&b.1).reverse().then(a.0.cmp(&b.0)));
    match leaderboard.single() {
        Ok(entity) => {
            commands
                .entity(entity)
                .despawn_related::<Children>()
                .with_children(|parent| {
                    players.iter().for_each(|(port, deaths)| {
                        parent.spawn(Text::new(format!("Player {}:", port)));
                        parent.spawn(Text::new(format!("{}", deaths)));
                    });
                });
        }
        Err(err) => {
            log::error!("There should only be a single leaderboard:\n{err}");
            leaderboard.iter().for_each(|entity| {
                commands.entity(entity).despawn();
            });
            setup_leaderboard(commands);
        }
    }
}
