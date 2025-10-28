use avian2d::prelude::*;
use bevy::{camera::RenderTarget, core_pipeline::tonemapping::Tonemapping, prelude::*};

use crate::{
    camera::{build_render_target, HIGHRES_LAYER, LYRA_LAYER},
    game::{
        animation::AnimationConfig,
        lyra::{
            animation::{LyraAnimationPlugin, PlayerAnimationType, ANIMATION_FRAMES},
            controller::{
                CachedLinearVelocity, CharacterControllerBundle, CharacterControllerPlugin,
            },
            strand::LyraStrandPlugin,
        },
        Layers,
    },
    ldtk::{LdtkLevelParam, LevelExt},
    shared::GameState,
};

mod animation;
mod controller;
mod strand;

pub const LYRA_RESPAWN_EPSILON: f32 = 16.0;

pub struct LyraPlugin;

impl Plugin for LyraPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: do not let ldtk spawn lyra because that would require level select to select a
        // level adjacent to lyra's level in the ldtk file.
        app.add_plugins(CharacterControllerPlugin);
        app.add_plugins(LyraStrandPlugin);
        app.add_plugins(LyraAnimationPlugin);
        app.add_systems(OnEnter(GameState::InGame), spawn_lyra);
        app.add_systems(OnEnter(GameState::InGame), spawn_lyra_cam.after(spawn_lyra));
        app.add_systems(OnExit(GameState::InGame), despawn_lyra);
    }
}

#[derive(Component)]
pub struct Lyra;

pub fn spawn_lyra(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ldtk_level_param: LdtkLevelParam,
) {
    info!("Spawning Lyra!");

    let Some(lyra_transform) = ldtk_level_param.cur_level().and_then(|level| {
        level
            .raw()
            .start_flag_pos()
            .map(|pos| Vec2::new(pos.x, pos.y + LYRA_RESPAWN_EPSILON))
    }) else {
        panic!("Current level must exist and it must have a start flag");
    };

    // NOTE: actual z value doesn't matter because lyra is rendered on a separate layer
    let player = commands
        .spawn(Lyra)
        .insert(Transform::from_translation(lyra_transform.extend(0.)))
        .id();

    let texture_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(15, 20),
        ANIMATION_FRAMES as u32,
        1,
        None,
        None,
    ));

    // insert sprite here because it depends on texture atlas which needs a resource
    commands
        .entity(player)
        .insert(Sprite {
            image: asset_server.load("lyra_sheet.png"),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            ..default()
        })
        .insert(LYRA_LAYER);

    commands
        .entity(player)
        .insert(CollisionLayers::new(
            Layers::PlayerCollider,
            [Layers::Terrain],
        ))
        .insert(CharacterControllerBundle::new(Collider::compound(vec![(
            Vec2::new(0.0, -2.0),
            Rotation::default(),
            Collider::rectangle(12.0, 16.0),
        )])))
        .insert(CachedLinearVelocity(Vec2::ZERO))
        .insert(PlayerAnimationType::Idle)
        .insert(AnimationConfig::from(PlayerAnimationType::Idle));

    commands
        .spawn(Collider::compound(vec![(
            Vec2::new(0.0, -2.0),
            Rotation::default(),
            Collider::rectangle(8.0, 10.0),
        )]))
        .insert(Sensor)
        .insert(ChildOf(player))
        .insert(RigidBody::Static)
        .insert(GravityScale(0.0))
        // .insert(PlayerHurtMarker)
        .insert(Transform::default())
        .insert(CollisionLayers::new(
            Layers::PlayerHurtbox,
            [Layers::DangerBox, Layers::Terrain, Layers::CrystalShard],
        ));
    // .insert(LineLight2d::point(
    //     Vec4::new(1.0, 1.0, 1.0, 1.0),
    //     40.0,
    //     0.01,
    // ));
}

#[derive(Component)]
pub struct PlayerCamera;

pub fn spawn_lyra_cam(
    lyra: Single<Entity, With<Lyra>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let (lyra_image, lyra_projection) = build_render_target(32, 32);
    let lyra_handle = images.add(lyra_image);

    // NOTE: lyra cam doesn't have pixelperfectcam because childing it to lyra makes it snap
    // automatically, and the canvas as a child means it also snaps automatically
    commands
        .spawn(Camera2d)
        .insert(PlayerCamera)
        .insert(Camera {
            order: 0,
            target: RenderTarget::Image(lyra_handle.clone().into()),
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        })
        .insert(Tonemapping::TonyMcMapface)
        .insert(lyra_projection)
        .insert(Transform::default())
        .insert(LYRA_LAYER)
        .insert(ChildOf(*lyra))
        .with_child((
            Sprite::from_image(lyra_handle.clone()),
            HIGHRES_LAYER,
            Transform::from_xyz(0., 0., 5.),
        ));
}

pub fn despawn_lyra(mut commands: Commands, player: Single<Entity, With<Lyra>>) {
    info!("Despawning Lyra!");

    commands.entity(*player).despawn();
}
