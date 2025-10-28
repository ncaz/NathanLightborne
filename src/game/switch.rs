use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    callback::Callback,
    camera::{CameraControlType, CameraMoveEvent},
    game::{
        camera_op::{camera_position_from_level, follow_lyra, CAMERA_ANIMATION_SECS},
        lyra::Lyra,
        LevelSystems,
    },
    ldtk::{LdtkLevelParam, LevelExt},
    shared::{AnimationState, PlayState, ResetLevels},
};

pub struct SwitchLevelPlugin;

impl Plugin for SwitchLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            switch_level
                .after(follow_lyra)
                .in_set(LevelSystems::Simulation),
        );
    }
}

/// [`System`] that will run on [`Update`] to check if the Player has moved to another level. If
/// the player has, then a MoveCameraEvent is sent. After the animation is finished, the Camera
/// handling code will send a LevelSwitch event that will notify other systems to cleanup the
/// levels.
#[allow(clippy::too_many_arguments)]
pub fn switch_level(
    mut commands: Commands,
    lyra: Single<&Transform, With<Lyra>>,
    mut ldtk_level_param: LdtkLevelParam,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut next_anim_state: ResMut<NextState<AnimationState>>,
) {
    for level in ldtk_level_param
        .ldtk_param
        .project()
        .expect("Level data must exist")
        .json_data()
        .levels
        .iter()
    {
        let level_box = level.level_box();
        if !level_box.contains(lyra.translation.xy()) {
            continue;
        }
        if ldtk_level_param
            .cur_iid()
            .expect("Cur level must exist")
            .as_str()
            == level.iid
        {
            continue;
        }

        next_play_state.set(PlayState::Animating);
        next_anim_state.set(AnimationState::Switch);

        let cb = commands
            .spawn(())
            .observe(
                |event: On<Callback>,
                 mut commands: Commands,
                 mut next_game_state: ResMut<NextState<PlayState>>| {
                    next_game_state.set(PlayState::Playing);
                    commands.trigger(ResetLevels);
                    commands.entity(event.entity).despawn();
                },
            )
            .id();

        commands.trigger(CameraMoveEvent {
            to: camera_position_from_level(level_box, lyra.translation.xy()),
            variant: CameraControlType::Animated {
                duration: Duration::from_secs_f32(CAMERA_ANIMATION_SECS),
                callback_entity: Some(cb),
                ease_fn: EaseFunction::SineInOut,
            },
        });

        // let allowed_colors = level
        //     .iter_enums_field("AllowedColors")
        //     .expect("AllowedColors should be enum array level field.")
        //     .map(|color_str| color_str.into())
        //     .collect::<Vec<LightColor>>();
        *ldtk_level_param.level_selection = LevelSelection::iid(LevelIid::new(level.iid.clone()));
        break;
    }
}
