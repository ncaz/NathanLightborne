use bevy::prelude::*;

use crate::game::defs::{
    crystal::CrystalPlugin, level_completion::LevelCompletionPlugin,
    one_way_platform::OneWayPlatformPlugin, sensor::LightSensorPlugin, shard::CrystalShardPlugin,
    spikes::SpikesPlugin, terrain::TerrainPlugin, tooltip_sign::TooltipSignPlugin,
};

pub mod crystal;
pub mod level_completion;
mod merge_tile;
pub mod one_way_platform;
pub mod sensor;
pub mod shard;
mod spikes;
mod terrain;
pub mod tooltip_sign;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_plugins(CrystalPlugin);
        app.add_plugins(LightSensorPlugin);
        app.add_plugins(OneWayPlatformPlugin);
        app.add_plugins(SpikesPlugin);
        app.add_plugins(TooltipSignPlugin);
        app.add_plugins(CrystalShardPlugin);
        app.add_plugins(LevelCompletionPlugin);
    }
}

#[derive(Component)]
pub struct DangerBox;
