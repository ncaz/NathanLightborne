use avian2d::prelude::{Collider, CollisionLayers, Friction};
use bevy::{prelude::*, time::Stopwatch};
use bevy_ecs_ldtk::prelude::*;
use enum_map::EnumMap;

use crate::{
    asset::LoadResource,
    game::{
        defs::crystal::{CrystalColor, CrystalToggleEvent},
        light::{segments::simulate_light_sources, HitByLight, LightColor},
        lighting::LineLight2d,
        Layers, LevelSystems,
    },
    shared::ResetLevels,
};

pub struct LightSensorPlugin;

impl Plugin for LightSensorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SensorAssets>();
        app.load_resource::<SensorAssets>();
        app.register_ldtk_entity::<LightSensorBundle>("Sensor");
        app.add_observer(hydrate_sensor);
        app.add_observer(reset_light_sensors);
        app.add_systems(
            FixedUpdate,
            update_light_sensors
                .after(simulate_light_sources)
                .in_set(LevelSystems::Simulation),
        );
    }
}

#[derive(Resource, Asset, Reflect, Clone)]
#[reflect(Resource)]
pub struct SensorAssets {
    #[dependency]
    sensor_inner: Handle<Image>,
    #[dependency]
    sensor_outer: Handle<Image>,
    #[dependency]
    sensor_center: Handle<Image>,
}

impl FromWorld for SensorAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            sensor_inner: asset_server.load("sensor/sensor_inner.png"),
            sensor_outer: asset_server.load("sensor/sensor_outer.png"),
            sensor_center: asset_server.load("sensor/sensor_center.png"),
        }
    }
}

/// [`Component`] added to entities receptive to light. The
/// [`activation_timer`](LightSensor::activation_timer) should be initialized in the
/// `From<&EntityInstance>` implemenation for the [`LightSensorBundle`], if not default.
///
/// The [`Sprite`] on the entity containing a [`LightSensor`] refers to the center part of the
/// sprite, which will be colored depending on the light that hits it.
#[derive(Component, Debug)]
pub struct LightSensor {
    /// Stores the cumulative time light has been hitting the sensor
    pub cumulative_exposure: Stopwatch,
    /// Stores the amount of light stored in the sensor, from 0 to 1.
    pub meter: f32,
    /// Colors of light beams hitting the sensor
    pub hit_by: EnumMap<LightColor, bool>,
    /// Active state of the sensor
    pub is_active: bool,
    /// The color of the crystals to toggle
    pub toggle_color: CrystalColor,
    /// Meter's rate of change, per fixed timestep tick.
    rate: f32,
    /// Stored color used to animate the center of the sensor when the light no longer hits it
    stored_color: Color,
}

impl LightSensor {
    fn new(toggle_color: CrystalColor, millis: i32) -> Self {
        let rate = 1.0 / (millis as f32) * (1000.0 / 64.0);
        LightSensor {
            meter: 0.0,
            cumulative_exposure: Stopwatch::default(),
            hit_by: EnumMap::default(),
            is_active: false,
            toggle_color,
            rate,
            stored_color: Color::WHITE,
        }
    }

    fn reset(&mut self) {
        self.meter = 0.0;
        self.hit_by = EnumMap::default();
        self.is_active = false;
        self.cumulative_exposure.reset();
    }

    fn is_hit(&self) -> bool {
        self.hit_by.iter().any(|(_, hit_by_color)| *hit_by_color)
    }

    fn iter_hit_color(&self) -> impl Iterator<Item = LightColor> + '_ {
        self.hit_by
            .iter()
            .filter_map(|(color, hit_by_color)| if *hit_by_color { Some(color) } else { None })
    }
}

impl From<&EntityInstance> for LightSensor {
    fn from(entity_instance: &EntityInstance) -> Self {
        let toggle_color: CrystalColor = entity_instance
            .get_enum_field("toggle_color")
            .expect("toggle_color needs to be an enum field on all sensors")
            .into();

        let millis = *entity_instance
            .get_int_field("activation_time")
            .expect("activation_time needs to be a float field on all sensors");

        LightSensor::new(toggle_color, millis)
    }
}

pub type ButtonColor = Color;

impl From<CrystalColor> for ButtonColor {
    fn from(value: CrystalColor) -> Self {
        match value {
            CrystalColor::Pink => Color::srgb(1.5, 0.7, 1.0),
            CrystalColor::Red => Color::srgb(1.0, 0.0, 0.0),
            CrystalColor::White => Color::srgb(0.9, 0.9, 0.9),
            CrystalColor::Blue => Color::srgb(0.6, 1.1, 1.9),
        }
    }
}

pub fn hydrate_sensor(
    event: On<Add, LightSensor>,
    mut commands: Commands,
    q_sensors: Query<&LightSensor>,
    sensor_assets: Res<SensorAssets>,
) {
    if q_sensors.is_empty() {
        return;
    }

    let inner_sprite = Sprite::from_image(sensor_assets.sensor_inner.clone());
    let mut outer_sprite = Sprite::from_image(sensor_assets.sensor_outer.clone());
    let center_sprite = Sprite::from_image(sensor_assets.sensor_center.clone());

    let sensor = q_sensors
        .get(event.entity)
        .expect("How else does trigger work skull");

    outer_sprite.color = ButtonColor::from(sensor.toggle_color);

    commands
        .entity(event.entity)
        .insert(LineLight2d::point(
            outer_sprite.color.to_linear().to_vec3().extend(0.5),
            35.0,
            0.02,
        ))
        .insert(Collider::rectangle(8., 8.))
        .insert(CollisionLayers::new(
            Layers::LightSensor,
            [Layers::LightRay, Layers::WhiteRay, Layers::BlueRay],
        ))
        .insert(Friction::new(0.))
        .insert(center_sprite.clone())
        .with_children(|sensor| {
            sensor.spawn(inner_sprite.clone());
            sensor.spawn(outer_sprite.clone());
        })
        .observe(
            |event: On<HitByLight>, mut q_sensors: Query<&mut LightSensor>| {
                let Ok(mut sensor) = q_sensors.get_mut(event.entity) else {
                    return;
                };
                sensor.hit_by[event.color] = event.hit;
            },
        );
}

#[derive(Bundle, LdtkEntity)]
pub struct LightSensorBundle {
    #[from_entity_instance]
    light_sensor: LightSensor,
}

pub fn reset_light_sensors(_: On<ResetLevels>, mut q_sensors: Query<&mut LightSensor>) {
    for mut sensor in q_sensors.iter_mut() {
        sensor.reset()
    }
}

pub fn update_light_sensors(
    mut commands: Commands,
    mut q_sensors: Query<(Entity, &mut LightSensor, &mut Sprite)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    for (entity, mut sensor, mut sprite) in q_sensors.iter_mut() {
        let was_hit = sensor.is_hit();

        if was_hit {
            sensor.cumulative_exposure.tick(time.delta());

            // if the sensor was hit, update the stored color for the sensor
            let mut col = Vec3::ZERO;
            for color in sensor.iter_hit_color() {
                col += color.lighting_color() * 0.5;
            }
            col += Vec3::splat(0.6);
            sensor.stored_color = Color::srgb(col.x, col.y, col.z);
        }

        let juice = if was_hit { sensor.rate } else { -sensor.rate };
        sensor.meter += juice;

        let mut send_toggle = || {
            commands.trigger(CrystalToggleEvent {
                color: sensor.toggle_color,
            });
            commands.entity(entity).with_child((
                AudioPlayer::new(asset_server.load("sfx/button.wav")),
                PlaybackSettings::DESPAWN,
            ));
        };

        if sensor.meter > 1.0 {
            if !sensor.is_active {
                send_toggle();
                sensor.is_active = true;
            }
            sensor.meter = 1.0;
        } else if sensor.meter < 0.0 {
            if sensor.is_active {
                send_toggle();
                sensor.is_active = false;
            }
            sensor.meter = 0.0;
        }

        sprite.color = Color::WHITE.mix(&sensor.stored_color, sensor.meter);
    }
}
