use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    callback::Callback,
    camera::{CameraControlType, CameraMoveEvent, CameraZoomEvent, HIGHRES_LAYER},
    game::{
        animation::AnimationConfig,
        camera_op::{camera_position_from_level, camera_position_from_level_with_scale},
        dialogue::{Dialogue, DialogueAssets, DialogueEntry},
        lighting::LineLight2d,
        lyra::Lyra,
        LevelSystems,
    },
    ldtk::{LdtkLevelParam, LevelExt},
    shared::{AnimationState, PlayState},
};

pub struct CrucieraPlugin;

impl Plugin for CrucieraPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<LdtkCrucieraBundle>("Gala");
        app.add_observer(setup_cruciera);
        app.add_systems(
            FixedUpdate,
            check_start_cutscene.in_set(LevelSystems::Simulation),
        );
    }
}

#[derive(Component, Default)]
pub struct Cruciera {
    played_cutscene: bool,
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct LdtkCrucieraBundle {
    #[worldly]
    worldy: Worldly,
    cruciera: Cruciera,
}

pub fn setup_cruciera(
    event: On<Add, Cruciera>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(15, 23),
        3,
        1,
        None,
        None,
    ));

    // insert sprite here because it depends on texture atlas which needs a resource
    commands
        .entity(event.entity)
        .insert(AnimationConfig::new(0, 2, 5, true))
        .insert((
            Sprite {
                image: asset_server.load("gala_sheet.png"),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                }),
                ..default()
            },
            HIGHRES_LAYER,
        ))
        .with_child(LineLight2d::point(Vec4::new(1.0, 0.2, 0.2, 0.8), 40., 0.01));
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn check_start_cutscene(
    mut commands: Commands,
    cruciera: Single<(&GlobalTransform, &mut Cruciera)>,
    lyra: Single<&GlobalTransform, (With<Lyra>, Without<Cruciera>)>,
    ldtk_level_param: LdtkLevelParam,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut next_anim_state: ResMut<NextState<AnimationState>>,
    cur_game_state: Res<State<PlayState>>,
) {
    if *cur_game_state.get() == PlayState::Animating {
        return;
    }
    let (cruciera_transform, mut cruciera) = cruciera.into_inner();

    if cruciera_transform
        .translation()
        .xy()
        .distance(lyra.translation().xy())
        < 40.
        && !cruciera.played_cutscene
    {
        cruciera.played_cutscene = true;

        let after_zoom_in = commands.spawn_empty().observe(start_dialogue).id();

        const CUTSCENE_CAMERA_SCALE: f32 = 0.75;

        commands.trigger(CameraZoomEvent {
            scale: CUTSCENE_CAMERA_SCALE,
            variant: CameraControlType::Animated {
                duration: Duration::from_millis(500),
                ease_fn: EaseFunction::SineInOut,
                callback_entity: Some(after_zoom_in),
            },
        });

        let camera_pos = camera_position_from_level_with_scale(
            ldtk_level_param
                .cur_level()
                .expect("Cur level exists")
                .raw()
                .level_box(),
            lyra.translation().xy(),
            CUTSCENE_CAMERA_SCALE,
        );

        commands.trigger(CameraMoveEvent {
            to: camera_pos,
            variant: CameraControlType::Animated {
                duration: Duration::from_millis(500),
                ease_fn: EaseFunction::SineInOut,
                callback_entity: None,
            },
        });

        next_play_state.set(PlayState::Animating);
        next_anim_state.set(AnimationState::InputLocked);
    }
}

#[derive(Clone, Copy)]
pub enum DialoguePortrait {
    LyraNeutral,
    LyraHappy,
    LyraSad,
    Cruciera,
}

pub static LYRA_CRUCIERA_DIALOGUE: [(DialoguePortrait, &str); 9] = [
    (DialoguePortrait::LyraNeutral, "Did you call for me, Lady Cruciera?"),
    (DialoguePortrait::Cruciera, "Indeed I have, young Lyra. I have one final job for you."),
    (DialoguePortrait::LyraHappy, "A job? What would you like me to do, Lady Cruciera?"),
    (DialoguePortrait::Cruciera, "I ask that you bring back the Divine Prism we have granted those... foolish humans down below."),
    (DialoguePortrait::LyraSad, "You wish for me to retrieve the Divine Prism? But I thought that it was a gift to the humans? Without it, they'll be misguided..."),
    (DialoguePortrait::Cruciera, "It was a gift, but they were misguided even with it in their possession."),
    (DialoguePortrait::Cruciera, "Their greed has split the Prism into pieces, and such pieces have been scattered across their realm."),
    (DialoguePortrait::Cruciera, "The task falls upon you, young Lyra. Those who corrupted such a relic don't deserve to keep it."),
    (DialoguePortrait::LyraNeutral, "Very well, Lady Cruciera. I'll try my best to retrieve the pieces."),
    // (DialoguePortrait::Cruciera, "It won't be as easy as you think, but it will be a good chance for you to experience the full power of a goddess."),
    // (DialoguePortrait::Cruciera, "The prism is strong, yet volatile. Harness its powers well and pass its trials - that is the only way you will understand the true meaning of responsibility."),
];

pub fn start_dialogue(
    _: On<Callback>,
    mut commands: Commands,
    dialogue_assets: Res<DialogueAssets>,
) {
    let after_dialogue = commands.spawn_empty().observe(end_dialogue).id();

    commands.trigger(Dialogue {
        entries: LYRA_CRUCIERA_DIALOGUE
            .iter()
            .map(|(portrait, text)| DialogueEntry {
                text: text.to_string(),
                image: match portrait {
                    DialoguePortrait::LyraSad => dialogue_assets.lyra_sad.clone(),
                    DialoguePortrait::LyraHappy => dialogue_assets.lyra_happy.clone(),
                    DialoguePortrait::LyraNeutral => dialogue_assets.lyra_neutral.clone(),
                    DialoguePortrait::Cruciera => dialogue_assets.cruciera.clone(),
                },
            })
            .collect(),
        callback_entity: Some(after_dialogue),
        duration: Duration::from_millis(20),
    })
}

pub fn end_dialogue(
    _: On<Callback>,
    mut commands: Commands,
    ldtk_level_param: LdtkLevelParam,
    lyra: Single<&GlobalTransform, With<Lyra>>,
) {
    let camera_pos = camera_position_from_level(
        ldtk_level_param
            .cur_level()
            .expect("cur level exist")
            .raw()
            .level_box(),
        lyra.translation().xy(),
    );

    let after_zoom_out = commands.spawn_empty().observe(reset_state).id();

    commands.trigger(CameraMoveEvent {
        to: camera_pos,
        variant: CameraControlType::Animated {
            duration: Duration::from_millis(500),
            ease_fn: EaseFunction::SineInOut,
            callback_entity: Some(after_zoom_out),
        },
    });
    commands.trigger(CameraZoomEvent {
        scale: 1.,
        variant: CameraControlType::Animated {
            duration: Duration::from_millis(500),
            ease_fn: EaseFunction::SineInOut,
            callback_entity: None,
        },
    });
}

pub fn reset_state(_: On<Callback>, mut next_game_state: ResMut<NextState<PlayState>>) {
    next_game_state.set(PlayState::Playing);
}
