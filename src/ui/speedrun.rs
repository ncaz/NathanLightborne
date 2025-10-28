use bevy::{prelude::*, time::Stopwatch};

use crate::{
    shared::{GameState, PlayState},
    ui::UiFont,
    utils::hhmmss::Hhmmss,
};

pub struct SpeedrunTimerPlugin;

impl Plugin for SpeedrunTimerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpeedrunTimer>();
        app.add_systems(OnEnter(GameState::InGame), spawn_speedrun_timer);
        app.add_systems(OnExit(GameState::InGame), despawn_speedrun_timer);
        app.add_systems(
            Update,
            tick_speedrun_timer.run_if(in_state(PlayState::Playing)),
        );
    }
}

#[derive(Default, Resource)]
pub struct SpeedrunTimer {
    pub enabled: bool,
    timer: Stopwatch,
}

#[derive(Component)]
pub struct SpeedrunUiMarker;

pub fn spawn_speedrun_timer(
    mut commands: Commands,
    speedrun_timer: Res<SpeedrunTimer>,
    ui_font: Res<UiFont>,
) {
    if !speedrun_timer.enabled {
        return;
    };
    commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(32.)),
            ..default()
        })
        .with_child((
            Text::new(speedrun_timer.timer.elapsed().hhmmssxxx()),
            SpeedrunUiMarker,
            ui_font.text_font().with_font_size(36.),
        ));
}

pub fn tick_speedrun_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut speedrun_timer: ResMut<SpeedrunTimer>,
    speedrun_ui: Single<Entity, With<SpeedrunUiMarker>>,
) {
    speedrun_timer.timer.tick(time.delta());

    commands
        .entity(*speedrun_ui)
        .insert(Text::new(speedrun_timer.timer.elapsed().hhmmssxxx()));
}

pub fn despawn_speedrun_timer(
    mut commands: Commands,
    speedrun_ui: Option<Single<Entity, With<SpeedrunUiMarker>>>,
) {
    let Some(speedrun_ui) = speedrun_ui else {
        return;
    };
    commands.entity(*speedrun_ui).despawn();
}
