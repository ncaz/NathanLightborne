use bevy::prelude::*;

use follow::TargetFollowingPlugin;
use level_select::LevelSelectPlugin;
use pause::PausePlugin;
use settings::SettingsPlugin;
use start_menu::StartMenuPlugin;
use tooltip::TooltipPlugin;

pub mod follow;
pub mod level_select;
mod pause;
mod settings;
mod start_menu;
pub mod tooltip;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TargetFollowingPlugin)
            .add_plugins(TooltipPlugin)
            .add_plugins(PausePlugin)
            .add_plugins(StartMenuPlugin)
            .add_plugins(LevelSelectPlugin)
            .add_plugins(SettingsPlugin);
    }
}
