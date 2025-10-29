use bevy::prelude::*;

use crate::game::defs::{
    crystal::CrystalPlugin, one_way_platform::OneWayPlatformPlugin, sensor::LightSensorPlugin,
    terrain::TerrainPlugin,
};

pub mod crystal;
mod merge_tile;
pub mod one_way_platform;
pub mod sensor;
mod terrain;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_plugins(CrystalPlugin);
        app.add_plugins(LightSensorPlugin);
        app.add_plugins(OneWayPlatformPlugin);
    }
}

#[derive(Component)]
pub struct DangerBox;
