use bevy::prelude::*;

use crate::game::defs::{
    crystal::CrystalPlugin, sensor::LightSensorPlugin, terrain::TerrainPlugin,
};

mod crystal;
mod merge_tile;
mod sensor;
mod terrain;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_plugins(CrystalPlugin);
        app.add_plugins(LightSensorPlugin);
    }
}

#[derive(Component)]
pub struct DangerBox;
