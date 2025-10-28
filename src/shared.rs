use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    InGame,
    Ui,
    Loading,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::InGame)]
pub enum PlayState {
    #[default]
    Playing,
    Paused,
    Animating,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(PlayState = PlayState::Animating)]
pub enum AnimationState {
    #[default]
    Switch,
    Respawn,
    Shard,
    ShardDialogue, // FIXME: copied to shit LOL
    Cruciera,
    CrucieraDialogue,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Ui)]
pub enum UiState {
    #[default]
    LevelSelect,
    Settings,
    StartMenu,
}

#[derive(Event, PartialEq, Eq)]
pub enum ResetLevel {
    /// Sent to run systems that reset the player state on respawn. If you are trying to kill the
    /// player, use `KillPlayerEvent` instead
    Respawn,
    /// Sent to run systems that reset the level state on level switch
    Switching,
}
