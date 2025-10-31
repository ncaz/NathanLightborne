use bevy::{prelude::*, ui::UiSystems};

use crate::{camera::MainCamera, shared::GameState};

pub struct TargetFollowingPlugin;
impl Plugin for TargetFollowingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            follow_ws_position_targets
                .before(UiSystems::Layout)
                // .after(TransformSystems::Propagate)
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
    mut q_ws_target_nodes: Query<(Entity, &WorldSpacePositionTarget)>,
    mut q_node: Query<&mut Node>,
    ws_xforms: Query<&GlobalTransform>,
) {
    let (camera, camera_transform) = *camera;

    for (entity, ws_target) in q_ws_target_nodes.iter_mut() {
        let Ok(global_xform) = ws_xforms.get(**ws_target) else {
            continue;
        };
        let pos_ws = global_xform.translation() + ws_target.offset;
        let Some(mut p) = camera.world_to_ndc(camera_transform, pos_ws) else {
            continue;
        };
        let Ok(mut node) = q_node.get_mut(entity) else {
            continue;
        };
        p = (p + 1.) * 50.;
        node.bottom = Val::Percent(p.y);
        node.left = Val::Percent(p.x);
    }
}
