use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{player::kill::KillPlayerEvent, shared::ResetLevel};

use super::follow::WorldSpacePositionTarget;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_tooltip_despawns);
    }
}

#[derive(Component)]
pub struct Tooltip;

#[derive(Component)]
pub enum TooltipDespawnSetting {
    // Despawns a certain time after being shown
    Time(Timer),
    PlayerKill,
    LevelSwitch,
}

pub fn handle_tooltip_despawns(
    mut commands: Commands,
    mut q_tooltips: Query<(Entity, &mut TooltipDespawnSetting)>,
    mut ev_level_switch: EventReader<ResetLevel>,
    mut ev_kill_player: EventReader<KillPlayerEvent>,
    time: Res<Time>,
) {
    let player_killed = ev_kill_player.read().count() > 0;
    let level_switched = ev_level_switch.read().count() > 0;

    for (tooltip, mut tooltip_despawn) in q_tooltips.iter_mut() {
        let tooltip_despawn = &mut *tooltip_despawn;
        match tooltip_despawn {
            TooltipDespawnSetting::Time(timer) => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    commands.entity(tooltip).despawn_recursive();
                }
            }
            TooltipDespawnSetting::PlayerKill => {
                if player_killed {
                    commands.entity(tooltip).despawn_recursive();
                }
            }
            TooltipDespawnSetting::LevelSwitch => {
                if level_switched {
                    commands.entity(tooltip).despawn_recursive();
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub struct TooltipSpawner<'w, 's> {
    commands: Commands<'w, 's>,
    asset_server: Res<'w, AssetServer>,
}

impl TooltipSpawner<'_, '_> {
    pub fn spawn_tooltip(
        &mut self,
        tooltip: impl Into<String>,
        on_entity: Entity,
        offset: Vec3,
        despawn: TooltipDespawnSetting,
    ) -> Entity {
        let font = TextFont {
            font: self.asset_server.load("fonts/Outfit-Medium.ttf"),
            font_size: 18.0,
            ..default()
        };

        self.commands
            .spawn((
                Node {
                    max_width: Val::Vw(20.),
                    ..default()
                },
                Text::new(tooltip),
                font,
                Tooltip,
                WorldSpacePositionTarget {
                    target: on_entity,
                    offset,
                },
                despawn,
            ))
            .id()
    }
}
