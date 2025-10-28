use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;

/// [`Component`] that holds the position of the cursor, in world coordinates. You should query
/// for this [`Component`] if you need the cursor position to do something. Note that if your
/// system uses this component, it should be set to run after [`update_cursor_world_coords`] for
/// consistency.
#[derive(Component, Default)]
pub struct CursorWorldCoords {
    pub pos: Vec2,
}

pub fn init_cursor_world_coords(mut commands: Commands) {
    commands.spawn(CursorWorldCoords::default());
}

/// [`Update`] [`System`] that updates the world position of the cursor every frame, and stores it
/// in the [`CursorWorldCoords`] component.
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
