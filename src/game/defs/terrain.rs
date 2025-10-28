use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::{defs::merge_tile::spawn_merged_tiles, Layers, LevelSystems};

use super::merge_tile::MergedTile;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell_for_layer::<TerrainBundle>("Terrain", 1);
        app.add_systems(
            PreUpdate,
            spawn_merged_tiles::<Terrain>.in_set(LevelSystems::Processing),
        );
    }
}

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
        commands
            .insert(Collider::rectangle(extent.x, extent.y))
            .insert(
                // Occluder2d::new(extent.x, extent.y),
                // DustSurface::Wall,
                CollisionLayers::new(Layers::Terrain, [Layers::PlayerCollider, Layers::LightRay]),
            )
            .insert(RigidBody::Static)
            .insert(Transform::from_xyz(center.x, center.y, 0.));
    }

    fn compare_data(&self) -> Self::CompareData {
        // all walls are mergable
    }
}
