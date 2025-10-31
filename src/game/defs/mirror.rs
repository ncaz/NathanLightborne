use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::Layers;

pub struct MirrorPlugin;
impl Plugin for MirrorPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell_for_layer::<MirrorBundle>("Terrain", 16);
        app.add_observer(adjust_mirrors);
    }
}

#[derive(Default, Component)]
pub struct Mirror;

#[derive(Bundle, Default, LdtkIntCell)]
pub struct MirrorBundle {
    mirror: Mirror,
}

pub fn adjust_mirrors(event: On<Add, Mirror>, mut commands: Commands) {
    commands
        .entity(event.entity)
        .insert(Collider::rectangle(8.0, 8.0))
        .insert(CollisionLayers::new(
            Layers::Terrain,
            [
                Layers::LightRay,
                Layers::PlayerHurtbox,
                Layers::PlayerCollider,
                Layers::BlueRay,
                Layers::WhiteRay,
            ],
        ));
}
