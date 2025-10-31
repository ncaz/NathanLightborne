use crate::{
    asset::LoadResource,
    camera::{MainCamera, HIGHRES_LAYER},
    config::Config,
    shared::{GameState, ResetLevels},
};
use avian2d::prelude::RigidBody;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    pub ldtk_file: Handle<LdtkProject>,
    #[dependency]
    pub background: Handle<Image>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let config = world.resource::<Config>();

        Self {
            ldtk_file: asset_server.load(&config.level_config.level_path),
            background: asset_server.load("levels/background.png"),
        }
    }
}

pub struct LevelSetupPlugin;

impl Plugin for LevelSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin);
        app.register_type::<LevelAssets>();
        app.load_resource::<LevelAssets>();
        // NOTE: LevelSelection resource here should be overwritten by level select
        app.insert_resource(LevelSelection::index(16));
        app.insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            level_background: LevelBackground::Nonexistent,
            ..default()
        });
        app.add_systems(OnEnter(GameState::InGame), spawn_level);
        app.add_systems(OnEnter(GameState::InGame), spawn_background);
        app.add_systems(OnExit(GameState::InGame), despawn_level);
        app.add_systems(OnExit(GameState::InGame), despawn_background);
    }
}

#[derive(Component)]
pub struct LevelMarker;

#[derive(Component)]
pub struct LevelBackgroundMarker;

pub fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
    info!("Spawning Levels!");

    commands
        .spawn(LdtkWorldBundle {
            ldtk_handle: level_assets.ldtk_file.clone().into(),
            ..Default::default()
        })
        // NOTE: RigidBody must be on the parent entity for god knows what reason
        .insert(RigidBody::Static)
        .insert(LevelMarker);
}

pub fn spawn_background(
    mut commands: Commands,
    camera: Single<Entity, With<MainCamera>>,
    level_assets: Res<LevelAssets>,
) {
    info!("Spawning Background!");

    commands
        .spawn(Sprite::from_image(level_assets.background.clone()))
        .insert(HIGHRES_LAYER)
        .insert(LevelBackgroundMarker)
        .insert(Transform::from_xyz(0., 0., -5.))
        .insert(ChildOf(*camera));
}

pub fn despawn_level(mut commands: Commands, level_marker: Single<Entity, With<LevelMarker>>) {
    info!("Despawning Levels!");
    commands.trigger(ResetLevels);
    commands.entity(*level_marker).despawn();
}

pub fn despawn_background(
    mut commands: Commands,
    level_bg: Single<Entity, With<LevelBackgroundMarker>>,
) {
    info!("Despawning Background!");

    commands.entity(*level_bg).despawn();
}
