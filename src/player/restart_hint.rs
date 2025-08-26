use std::time::Duration;

use bevy::prelude::*;

use crate::{
    level::{CurrentLevel, LevelSystems},
    ui::tooltip::{TooltipDespawnSetting, TooltipSpawner},
};

use super::{light::PlayerLightInventory, PlayerMarker};

const PLAYER_STUCK_TOOLTIP_DELAY_SECS: u64 = 5;

pub struct HintRestartPlugin;

impl Plugin for HintRestartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hint_restart_button.in_set(LevelSystems::Simulation));
    }
}

#[derive(Deref, DerefMut)]
pub struct HintRestartTimer(Timer);

impl Default for HintRestartTimer {
    fn default() -> Self {
        Self(Timer::new(
            Duration::from_secs(PLAYER_STUCK_TOOLTIP_DELAY_SECS),
            TimerMode::Once,
        ))
    }
}

pub fn hint_restart_button(
    mut tooltip_spawner: TooltipSpawner,
    mut triggered: Local<HintRestartTimer>,
    q_player: Query<(Entity, &PlayerLightInventory), With<PlayerMarker>>,
    time: Res<Time>,
    current_level: Res<CurrentLevel>,
) {
    let Ok((player_entity, player_inventory)) = q_player.get_single() else {
        return;
    };

    let has_color = current_level
        .allowed_colors
        .iter()
        .any(|(_, allowed)| *allowed);

    let can_shoot = player_inventory
        .sources
        .iter()
        .any(|(color, has_shot)| current_level.allowed_colors[color] && *has_shot);

    if !can_shoot && has_color {
        triggered.tick(time.delta());
        if triggered.just_finished() {
            // pause timer so it doesn't continue even after reset
            triggered.pause();
            tooltip_spawner.spawn_tooltip(
                "Stuck? Press R to restart",
                player_entity,
                Vec3::new(0., 20., 0.),
                TooltipDespawnSetting::LevelSwitch,
            );
        }
    } else {
        triggered.reset();
    }
}
