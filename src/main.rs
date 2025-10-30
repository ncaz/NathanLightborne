use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
// use animation::SpriteAnimationPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::{asset::AssetMetaCheck, diagnostic::LogDiagnosticsPlugin};
// use bevy_rapier2d::prelude::*;

use camera::{CameraPlugin, HIGHRES_LAYER};
use config::ConfigPlugin;
// use input::{init_cursor_world_coords, update_cursor_world_coords};
// use level::LevelManagementPlugin;
// use light::LightManagementPlugin;
// use lighting::DeferredLightingPlugin;
// use particle::ParticlePlugin;
// use player::PlayerManagementPlugin;
use shared::{AnimationState, GameState, UiState};
use sound::SoundPlugin;
use ui::UiPlugin;

use crate::asset::{AssetLoadPlugin, ResourceLoaded};
use crate::game::GamePlugin;
use crate::shared::PlayState;

// mod animation;
mod asset;
mod callback;
mod camera;
mod config;
mod game;
mod ldtk;
// mod light;
// mod lighting;
// mod particle;
mod shared;
mod sound;
mod ui;
mod utils;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Lightborne".into(),
                    name: Some("lightborne".into()),
                    present_mode: PresentMode::AutoNoVsync,
                    canvas: Some("#bevy-container".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                //https://github.com/bevyengine/bevy_github_ci_template/issues/48
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    );
    app.insert_gizmo_config::<DefaultGizmoConfigGroup>(
        DefaultGizmoConfigGroup,
        GizmoConfig {
            enabled: true,
            render_layers: HIGHRES_LAYER,
            ..Default::default()
        },
    );
    app.add_plugins(AssetLoadPlugin);
    app.add_plugins(ConfigPlugin);
    app.add_plugins(LogDiagnosticsPlugin::default());
    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    // app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(8.0).in_fixed_schedule());
    // app.add_plugins(SpriteAnimationPlugin);
    // app.add_plugins(PlayerManagementPlugin);
    // app.add_plugins(LevelManagementPlugin);
    // app.add_plugins(LightManagementPlugin);
    app.add_plugins(SoundPlugin);
    // app.add_plugins(ParticlePlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(UiPlugin);
    app.add_plugins(GamePlugin);
    app.insert_state(GameState::Loading);
    app.add_sub_state::<UiState>();
    app.add_sub_state::<PlayState>();
    app.add_sub_state::<AnimationState>();
    // app.add_plugins(DeferredLightingPlugin);

    app.add_observer(
        |event: On<ResourceLoaded>,
         mut next_game_state: ResMut<NextState<GameState>>,
         mut next_ui_state: ResMut<NextState<UiState>>| match *event {
            ResourceLoaded::Finished => {
                info!("Resources Loaded");
                next_game_state.set(GameState::Ui);
                next_ui_state.set(UiState::StartMenu);
            }
            ResourceLoaded::InProgress { finished, waiting } => {
                info!(
                    "Resources Loading... Waiting: {} Finished: {}",
                    waiting, finished
                );
            }
        },
    );

    app.run();
}
