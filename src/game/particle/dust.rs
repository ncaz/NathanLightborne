use std::time::Duration;

use avian2d::prelude::{LinearVelocity, ShapeHits};
use bevy::prelude::*;
use bevy::time::Stopwatch;
use rand::{self, seq::IndexedRandom};

use crate::game::{
    defs::{
        crystal::{CrystalColor, CrystalGroup},
        sensor::ButtonColor,
    },
    lyra::{controller::Grounded, Lyra},
};

use super::{ParticleBundle, ParticleOptions, ParticlePhysicsOptions};

#[derive(Resource, Reflect, Asset, Clone)]
#[reflect(Resource)]
pub struct DustAssets {
    #[dependency]
    wall: [Handle<Image>; 2],
    #[dependency]
    wood: [Handle<Image>; 3],
    #[dependency]
    crystal: [Handle<Image>; 4],
}

impl FromWorld for DustAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            wall: [
                asset_server.load("particle/wall_dust_1.png"),
                asset_server.load("particle/wall_dust_2.png"),
            ],
            wood: [
                asset_server.load("particle/wood_dust_1.png"),
                asset_server.load("particle/wood_dust_2.png"),
                asset_server.load("particle/wood_dust_3.png"),
            ],
            crystal: [
                asset_server.load("particle/crystal_dust_1.png"),
                asset_server.load("particle/crystal_dust_2.png"),
                asset_server.load("particle/crystal_dust_3.png"),
                asset_server.load("particle/crystal_dust_4.png"),
            ],
        }
    }
}

#[derive(Component, Debug)]
pub enum DustSurface {
    Wall,
    Wood,
    Crystal(CrystalColor),
}

impl DustSurface {
    fn new_particle_options(
        &self,
        starting_velocity: Vec2,
        assets: &Res<DustAssets>,
    ) -> ParticleOptions {
        let mut rng = rand::rng();
        let gravity_mult = match self {
            Self::Wall => 200.0,
            Self::Wood => 120.0,
            Self::Crystal(_) => 200.0,
        };

        let images: &[Handle<Image>] = match self {
            Self::Wall => &assets.wall,
            Self::Wood => &assets.wood,
            Self::Crystal(_) => &assets.crystal,
        };
        let color = if let Self::Crystal(color) = self {
            ButtonColor::from(*color)
        } else {
            Color::default()
        };
        let life_time = (2.0 * starting_velocity.y) / gravity_mult;
        ParticleOptions {
            life_time: Duration::from_secs_f32(life_time),
            physics: Some(ParticlePhysicsOptions {
                wind_mult: 0.0,
                gravity_mult,
                starting_velocity,
            }),
            animation: None,
            sprite: Sprite {
                image: images.choose(&mut rng).unwrap().clone(),
                color,
                ..default()
            },
            ..default()
        }
    }

    fn spawn_interval(&self) -> Duration {
        Duration::from_secs_f32(match self {
            Self::Wall => 0.05,
            Self::Wood => 0.05,
            Self::Crystal(_) => 0.06,
        })
    }

    fn splash_amount(&self) -> usize {
        match self {
            Self::Wall => 7,
            Self::Wood => 6,
            Self::Crystal(_) => 4,
        }
    }

    fn new_spawn_pos_from_player_pos(&self, player_pos: Vec2) -> Vec2 {
        player_pos + Vec2::new(0.0, -10.0) + Vec2::new(rand::random_range(-4.0..4.0), 0.0)
    }

    fn new_starting_velocity(&self) -> Vec2 {
        match self {
            Self::Wall => Vec2::new(
                rand::random_range(-1.0..1.0) * 20.0,
                rand::random_range(10.0..40.0),
            ),
            Self::Wood => Vec2::new(
                rand::random_range(-1.0..1.0) * 20.0,
                rand::random_range(10.0..30.0),
            ),
            Self::Crystal(_) => Vec2::new(
                rand::random_range(-1.0..1.0) * 20.0,
                rand::random_range(10.0..40.0),
            ),
        }
    }
}

#[derive(Resource, Default)]
pub struct DustSpawnStopwatch {
    pub walking: Stopwatch,
    pub landing: Stopwatch,
}

pub fn add_crystal_dust(
    mut commands: Commands,
    crystals: Query<(Entity, &CrystalGroup), Added<CrystalGroup>>,
) {
    for (entity, crystal) in crystals.iter() {
        commands
            .entity(entity)
            .insert(DustSurface::Crystal(crystal.0.color));
    }
}

pub fn spawn_player_walking_dust(
    mut commands: Commands,
    lyra: Single<(&Transform, &LinearVelocity, &ShapeHits, Has<Grounded>), With<Lyra>>,
    dust_assets: Res<DustAssets>,
    dust_surfaces: Query<&DustSurface>,
    mut dust_spawn_stopwatch: ResMut<DustSpawnStopwatch>,
    time: Res<Time>,
    mut was_grounded: Local<bool>,
) {
    dust_spawn_stopwatch.walking.tick(time.delta());
    dust_spawn_stopwatch.landing.tick(time.delta());

    let (player_t, lin_vel, shape_hits, is_grounded) = lyra.into_inner();
    if !is_grounded {
        *was_grounded = is_grounded;
        return;
    }

    let Some(dust_surface) = shape_hits
        .iter()
        .find_map(|hit| dust_surfaces.get(hit.entity).ok())
    else {
        return;
    };

    let (particle_spawn_amount, velocity_mult) = match (*was_grounded, lin_vel.0.length()) {
        // if at walking speed, spawn one
        (true, 80.0..) => (
            if dust_spawn_stopwatch.walking.elapsed() > dust_surface.spawn_interval() {
                dust_spawn_stopwatch.walking.reset();
                1
            } else {
                0
            },
            1.0,
        ),
        (false, ..) => (
            if dust_spawn_stopwatch.landing.elapsed() > Duration::from_secs_f32(0.1) {
                dust_spawn_stopwatch.landing.reset();
                dust_surface.splash_amount()
            } else {
                0
            },
            2.0,
        ),
        _ => (0, 0.0),
    };

    for _ in 0..particle_spawn_amount {
        let pos = dust_surface.new_spawn_pos_from_player_pos(player_t.translation.truncate());

        let starting_velocity = dust_surface.new_starting_velocity() * velocity_mult;

        commands.spawn(ParticleBundle::new(
            dust_surface.new_particle_options(starting_velocity, &dust_assets),
            pos,
        ));
    }
    *was_grounded = is_grounded;
}
