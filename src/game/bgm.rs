use bevy::prelude::*;

use crate::{
    game::LevelSystems,
    ldtk::{LdtkLevelParam, LevelExt},
    sound::{BgmTrack, ChangeBgmEvent},
};

pub struct LevelBgmPlugin;

impl Plugin for LevelBgmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            set_bgm_from_current_level.in_set(LevelSystems::Simulation),
        );
    }
}

pub fn set_bgm_from_current_level(
    mut commands: Commands,
    ldtk_level_param: LdtkLevelParam,
    mut old_bgm: Local<BgmTrack>,
) {
    let cur_id = ldtk_level_param
        .cur_level()
        .expect("Cur level should exist")
        .raw()
        .level_id();

    let new_bgm = match cur_id {
        val if &val[0..1] == "2" || &val[0..1] == "1" => BgmTrack::MustntStop,
        val if &val[0..1] == "3" => BgmTrack::Cutscene1Draft,
        val if &val[0..1] == "4" => BgmTrack::LightInTheDark,
        _ => BgmTrack::None,
    };

    if new_bgm != *old_bgm {
        commands.trigger(ChangeBgmEvent(new_bgm));
        *old_bgm = new_bgm;
    }
}
