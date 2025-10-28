use avian2d::prelude::PhysicsSystems;
use bevy::prelude::*;

use crate::{
    camera::{CameraControlType, CameraMoveEvent, MainCamera, CAMERA_HEIGHT, CAMERA_WIDTH},
    game::{
        lyra::{spawn_lyra, Lyra},
        LevelSystems,
    },
    ldtk::{LdtkLevelParam, LevelExt},
    shared::GameState,
};

pub struct CameraOpPlugin;

impl Plugin for CameraOpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), follow_lyra.after(spawn_lyra));
        app.add_systems(
            FixedPostUpdate,
            follow_lyra
                .in_set(LevelSystems::Simulation)
                .after(PhysicsSystems::Writeback),
        );
    }
}

pub fn follow_lyra(
    mut commands: Commands,
    lyra: Single<&Transform, With<Lyra>>,
    camera: Single<&Transform, (With<MainCamera>, Without<Lyra>)>,
    ldtk_level_param: LdtkLevelParam,
) {
    let cur_level = ldtk_level_param
        .cur_level()
        .expect("Current level should exist!")
        .raw();

    let camera_pos = camera_position_from_level(cur_level.level_box(), lyra.translation.xy());
    commands.trigger(CameraMoveEvent {
        to: camera.translation.xy().lerp(camera_pos, 0.2),
        variant: CameraControlType::Instant,
    });
}

pub fn camera_position_from_level_with_scale(
    level_box: Rect,
    player_pos: Vec2,
    camera_scale: f32,
) -> Vec2 {
    let (x_min, x_max) = (
        level_box.min.x + CAMERA_WIDTH as f32 * 0.5 * camera_scale,
        level_box.max.x - CAMERA_WIDTH as f32 * 0.5 * camera_scale,
    );
    let (y_min, y_max) = (
        level_box.min.y + CAMERA_HEIGHT as f32 * 0.5 * camera_scale,
        level_box.max.y - CAMERA_HEIGHT as f32 * 0.5 * camera_scale,
    );

    Vec2::new(
        player_pos.x.max(x_min).min(x_max),
        player_pos.y.max(y_min).min(y_max),
    )
}

pub fn camera_position_from_level(level_box: Rect, player_pos: Vec2) -> Vec2 {
    camera_position_from_level_with_scale(level_box, player_pos, 1.)
}
