use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;

pub struct CursorCoordsPlugin;

impl Plugin for CursorCoordsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_cursor_world_coords);
        app.add_systems(PreUpdate, update_cursor_world_coords);
    }
}

#[derive(Component, Default)]
pub struct CursorWorldCoords {
    pub pos: Vec2,
}

pub fn init_cursor_world_coords(mut commands: Commands) {
    commands.spawn(CursorWorldCoords::default());
}

pub fn update_cursor_world_coords(
    mut q_coords: Query<&mut CursorWorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };
    let Ok(window) = q_window.single() else {
        return;
    };
    let Ok(mut world_coords) = q_coords.single_mut() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(cursor_ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };
    world_coords.pos = cursor_ray.origin.truncate();
}
