use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::{
    defs::{
        merge_tile::spawn_merged_tiles,
        terrain::{Terrain, TerrainBundle},
    },
    LevelSystems,
};

mod merge_tile;
mod terrain;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell_for_layer::<TerrainBundle>("Terrain", 1);
        // app.register_ldtk_int_cell_for_layer::<SpikeBundle>("Terrain", 2);
        app.add_systems(
            PreUpdate,
            spawn_merged_tiles::<Terrain>.in_set(LevelSystems::Processing),
        );
    }
}
