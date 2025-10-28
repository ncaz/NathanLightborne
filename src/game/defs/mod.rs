use bevy::prelude::*;

use crate::game::defs::{crystal::CrystalPlugin, terrain::TerrainPlugin};

mod crystal;
mod merge_tile;
mod terrain;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_plugins(CrystalPlugin);
    }
}

#[derive(Component)]
pub struct DangerBox;
