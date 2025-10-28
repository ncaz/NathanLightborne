use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::Layers;

use super::merge_tile::MergedTile;

#[derive(Default, Component)]
pub struct Terrain;

#[derive(Default, Bundle, LdtkIntCell)]
pub struct TerrainBundle {
    terrain: Terrain,
}

impl MergedTile for Terrain {
    type CompareData = ();

    fn bundle(
        commands: &mut EntityCommands,
        center: Vec2,
        extent: Vec2,
        _compare_data: &Self::CompareData,
    ) {
        commands.insert((
            Collider::rectangle(extent.x, extent.y),
            // Occluder2d::new(extent.x, extent.y),
            CollisionLayers::new(
                Layers::Terrain,
                [Layers::PlayerCollider, Layers::LightRay, Layers::Strand],
            ),
            RigidBody::Static,
            Transform::from_xyz(center.x, center.y, 0.),
            // DustSurface::Wall,
        ));
    }

    fn compare_data(&self) -> Self::CompareData {
        // all walls are mergable
    }
}
