use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    asset::LoadResource,
    callback::Callback,
    camera::{CameraTransition, CameraTransitionEvent},
    game::{
        camera_op::SnapToLyra,
        lyra::{lyra_spawn_transform, Lyra},
    },
    ldtk::LdtkLevelParam,
    shared::{AnimationState, PlayState, ResetLevels, ResetPlayer},
};

pub struct LyraKillPlugin;

impl Plugin for LyraKillPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KillSfx>();
        app.load_resource::<KillSfx>();
        app.add_observer(reset_player_pos_on_kill);
        app.add_observer(start_kill_animation);
        app.add_observer(play_death_sound_on_kill);
        app.add_systems(
            Update,
            quick_reset
                .run_if(input_just_pressed(KeyCode::KeyR))
                .run_if(in_state(PlayState::Playing)),
        );
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct KillSfx {
    sfx: Handle<AudioSource>,
}

impl FromWorld for KillSfx {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            sfx: asset_server.load("sfx/death.wav"),
        }
    }
}

pub fn quick_reset(mut commands: Commands) {
    commands.trigger(KillPlayer);
}

pub fn play_death_sound_on_kill(
    _: On<KillPlayer>,
    mut commands: Commands,
    lyra: Single<Entity, With<Lyra>>,
    kill_sfx: Res<KillSfx>,
) {
    commands.entity(*lyra).with_child((
        AudioPlayer::new(kill_sfx.sfx.clone()),
        PlaybackSettings::DESPAWN,
    ));
}

pub fn reset_player_pos_on_kill(
    _: On<ResetPlayer>,
    mut commands: Commands,
    mut lyra: Single<&mut Transform, With<Lyra>>,
    ldtk_level_param: LdtkLevelParam,
) {
    let lyra_transform = lyra_spawn_transform(&ldtk_level_param);
    **lyra = Transform::from_translation(lyra_transform.extend(0.));
    commands.trigger(SnapToLyra);
}

#[derive(Event)]
pub struct KillPlayer;

pub fn start_kill_animation(
    _: On<KillPlayer>,
    mut commands: Commands,
    mut next_game_state: ResMut<NextState<PlayState>>,
    mut next_anim_state: ResMut<NextState<AnimationState>>,
) {
    let cb1 = commands
        .spawn(())
        .observe(|_: On<Callback>, mut commands: Commands| {
            let cb2 = commands
                .spawn(())
                .observe(
                    |_: On<Callback>, mut next_play_state: ResMut<NextState<PlayState>>| {
                        next_play_state.set(PlayState::Playing);
                    },
                )
                .id();

            commands.trigger(CameraTransitionEvent {
                duration: Duration::from_millis(400),
                ease_fn: EaseFunction::SineInOut,
                callback_entity: Some(cb2),
                effect: CameraTransition::SlideFromBlack,
            });
            commands.trigger(ResetLevels);
            commands.trigger(ResetPlayer);
        })
        .id();

    commands.trigger(CameraTransitionEvent {
        duration: Duration::from_millis(400),
        ease_fn: EaseFunction::SineInOut,
        callback_entity: Some(cb1),
        effect: CameraTransition::SlideToBlack,
    });

    next_game_state.set(PlayState::Animating);
    next_anim_state.set(AnimationState::InputLocked);
}
