use bevy::{prelude::*, ui::UiSystems, window::PrimaryWindow};

use crate::{camera::MainCamera, shared::GameState};

pub struct TargetFollowingPlugin;
impl Plugin for TargetFollowingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            follow_ws_position_targets
                .after(UiSystems::Layout)
                .before(TransformSystems::Propagate)
                .run_if(in_state(GameState::InGame)),
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
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut q_ws_target_nodes: Query<(Entity, &WorldSpacePositionTarget)>,
    mut xforms: ParamSet<(TransformHelper, Query<&mut Transform>)>,
) {
    let (camera, camera_transform) = *camera;

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
        p *= window.scale_factor();

        t.translation = p.extend(t.translation.z);
    }
}
