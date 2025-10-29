use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::systems::process_ldtk_levels;

use crate::{
    camera::HIGHRES_LAYER,
    game::{
        animation::SpriteAnimationPlugin, camera_op::CameraOpPlugin, cursor::CursorCoordsPlugin,
        defs::LevelPlugin, light::LightBeamPlugin, lighting::DeferredLightingPlugin,
        lyra::LyraPlugin, setup::LevelSetupPlugin, switch::SwitchLevelPlugin,
    },
    shared::{GameState, PlayState},
};

mod animation;
mod camera_op;
mod cursor;
mod defs;
pub mod light;
pub mod lyra;
pub mod setup;
mod switch;
// mod input;
// mod level;
// mod light;
pub mod lighting;
// mod particle;
// mod player;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default().with_length_unit(8.));
        app.add_plugins(PhysicsDebugPlugin);
        app.add_plugins(CursorCoordsPlugin);
        app.add_plugins(SpriteAnimationPlugin);
        app.add_plugins(SwitchLevelPlugin);
        app.add_plugins(LevelSetupPlugin);
        app.add_plugins(LyraPlugin);
        app.add_plugins(LevelPlugin);
        app.add_plugins(CameraOpPlugin);
        app.add_plugins(LightBeamPlugin);
        app.add_plugins(DeferredLightingPlugin);
        app.configure_sets(
            PreUpdate,
            LevelSystems::Processing
                .after(process_ldtk_levels)
                .run_if(in_state(GameState::InGame)),
        );
        app.configure_sets(
            Update,
            LevelSystems::Simulation.run_if(in_state(PlayState::Playing)),
        );
        app.configure_sets(
            FixedUpdate,
            LevelSystems::Simulation.run_if(in_state(PlayState::Playing)),
        );
        app.insert_gizmo_config(
            PhysicsGizmos::default(),
            GizmoConfig {
                enabled: true,
                render_layers: HIGHRES_LAYER,
                ..Default::default()
            },
        );
    }
}

#[derive(PhysicsLayer, Default)]
pub enum Layers {
    #[default]
    Default,
    PlayerCollider,
    PlayerHurtbox,
    DangerBox,
    Terrain,
    LightRay,
    LightSensor,
    WhiteRay,
    BlueRay,
    CrystalShard,
    // BlackRay,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LevelSystems {
    /// Systems used to simulate game logic
    Simulation,
    /// Non-trigger systems that are used to process ldtk entities. Prefer to use triggers.
    Processing,
}
