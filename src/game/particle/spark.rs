use std::time::Duration;

use bevy::prelude::*;

use crate::game::{
    light::segments::LightSegment,
    particle::{
        emitter::ParticleModifier, ParticleBundle, ParticleEmitter, ParticleEmitterArea,
        ParticleEmitterOptions, ParticleOptions, ParticlePhysicsOptions,
    },
};

#[derive(Resource, Asset, Clone, Reflect)]
pub struct SparkAssets {
    #[dependency]
    spark: Handle<Image>,
}

impl FromWorld for SparkAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            spark: asset_server.load("particle/spark.png"),
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn add_segment_sparks(
    event: On<Add, LightSegment>,
    mut commands: Commands,
    light_segment: Query<&LightSegment>,
    spark_assets: Res<SparkAssets>,
) {
    const VEL: f32 = 30.0;

    let Ok(segment) = light_segment.get(event.entity) else {
        return;
    };

    commands
        .entity(event.entity)
        .insert(ParticleEmitter::new(ParticleEmitterOptions {
            area: ParticleEmitterArea::Capsule { radius: 1.0 },
            delay_range: Duration::from_secs_f32(0.0)..Duration::from_secs_f32(500.0),
            scale_delay_by_area: true,
            particles: vec![new_spark_particle(
                segment.color.light_beam_color(),
                &spark_assets,
            )],
            modifier: ParticleModifier {
                add_velocity: Some((-VEL..VEL, -VEL..VEL)),
            },
        }))
        .with_child((
            ParticleEmitter::new(ParticleEmitterOptions {
                area: ParticleEmitterArea::Circle { radius: 0.5 },
                delay_range: Duration::from_secs_f32(0.0)..Duration::from_secs_f32(0.4),
                particles: vec![new_spark_particle(
                    segment.color.light_beam_color(),
                    &spark_assets,
                )],
                modifier: ParticleModifier {
                    add_velocity: Some((-VEL..VEL, -VEL..VEL)),
                },
                ..default()
            }),
            Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)), // moves child emitter to the end of segment (due to scale)
        ));
}

#[derive(Message)]
pub struct SparkExplosionEvent {
    pub pos: Vec2,
    pub color: Color,
}

pub fn create_spark_explosions(
    mut commands: Commands,
    mut spark_explosion_events: MessageReader<SparkExplosionEvent>,
    spark_assets: Res<SparkAssets>,
) {
    const VEL: f32 = 50.0;
    let modifier: ParticleModifier = ParticleModifier {
        add_velocity: Some((-VEL..VEL, -VEL..VEL)),
    };
    for event in spark_explosion_events.read() {
        for _ in 0..15 {
            let SparkExplosionEvent { pos, color } = *event;
            let mut particle_options = new_spark_particle(color, &spark_assets);
            modifier.modify(&mut particle_options);
            commands.spawn(ParticleBundle::new(particle_options, pos));
        }
    }
}

fn new_spark_particle(color: Color, spark_assets: &Res<SparkAssets>) -> ParticleOptions {
    ParticleOptions {
        life_time: Duration::from_secs_f32(0.6),
        physics: Some(ParticlePhysicsOptions {
            wind_mult: 0.0,
            gravity_mult: 200.0,
            starting_velocity: Vec2::new(0.0, 10.0),
        }),
        sprite: Sprite {
            image: spark_assets.spark.clone(),
            color,
            ..default()
        },
        fade_away: true,
        ..default()
    }
}
