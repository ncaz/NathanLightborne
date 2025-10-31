use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::{defs::DangerBox, Layers};

pub struct SpikesPlugin;

impl Plugin for SpikesPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell_for_layer::<Spike>("Terrain", 2);
        app.add_observer(on_add_spikes);
    }
}

#[derive(Component, LdtkIntCell)]
pub struct Spike {}

pub fn on_add_spikes(event: On<Add, Spike>, mut commands: Commands) {
    commands
        .entity(event.entity)
        .insert(Collider::triangle(
            Vec2::new(-4., -4.),
            Vec2::new(4., -4.),
            Vec2::new(0., 4.),
        ))
        .insert(DangerBox)
        .insert(CollisionLayers::new(
            [Layers::DangerBox, Layers::Spike],
            [
                Layers::PlayerHurtbox,
                Layers::LightRay,
                Layers::BlueRay,
                Layers::WhiteRay,
            ],
        ));
}
