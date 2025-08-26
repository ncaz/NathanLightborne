use bevy::{prelude::*, window::PrimaryWindow};

use crate::camera::MainCamera;

pub struct TargetFollowingPlugin;
impl Plugin for TargetFollowingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            follow_ws_position_targets
                .after(bevy::ui::UiSystem::Layout)
                .before(bevy::transform::TransformSystem::TransformPropagate),
        );
    }
}

// Add this component to a UI element that should "follow" some entity in world-space
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct WorldSpacePositionTarget {
    #[deref]
    pub target: Entity,
    pub offset: Vec3,
}

fn follow_ws_position_targets(
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut q_ws_target_nodes: Query<(Entity, &WorldSpacePositionTarget)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut xforms: ParamSet<(TransformHelper, Query<&mut Transform>)>,
) {
    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };
    let Ok(window) = q_window.get_single() else {
        return;
    };

    for (entity, ws_target) in q_ws_target_nodes.iter_mut() {
        let Ok(global_xform) = xforms.p0().compute_global_transform(**ws_target) else {
            continue;
        };
        let pos_ws = global_xform.translation() + ws_target.offset;
        let Ok(mut p) = camera.world_to_viewport(camera_transform, pos_ws) else {
            continue;
        };

        let mut xforms = xforms.p1();
        let Ok(mut t) = xforms.get_mut(entity) else {
            continue;
        };
        // Convert physical pixels -> logical UI units.
        p *= window.scale_factor() as f32;

        t.translation = p.extend(t.translation.z);
    }
}
