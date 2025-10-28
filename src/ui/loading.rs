use bevy::prelude::*;

use crate::{asset::ResourceLoaded, shared::GameState, ui::UiFontSize};

pub struct LoadingUiPlugin;

impl Plugin for LoadingUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), spawn_loading_ui)
            .add_observer(update_loading_ui)
            .add_systems(OnExit(GameState::Loading), despawn_loading_ui);
    }
}

#[derive(Component)]
pub struct LoadingMenuUi;

#[derive(Component)]
pub struct LoadingMenuProgress;

pub fn spawn_loading_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // NOTE: cannot use UiFont resource because it has not been loaded yet
    let font = TextFont {
        font: asset_server.load("fonts/Outfit-Medium.ttf"),
        font_size: UiFontSize::HEADER,
        ..default()
    };

    let container = commands
        .spawn(LoadingMenuUi)
        .insert(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(96.)),
            row_gap: Val::Px(48.),
            ..default()
        })
        .insert(BackgroundColor(Color::BLACK))
        .id();

    commands
        .spawn(Text::new("Loading..."))
        .insert(ChildOf(container))
        .insert(font);

    let progress_container = commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Px(12.),
            ..default()
        })
        .insert(ChildOf(container))
        .id();

    commands
        .spawn(Node {
            width: Val::Percent(0.),
            height: Val::Percent(100.),
            ..default()
        })
        .insert(LoadingMenuProgress)
        .insert(ChildOf(progress_container))
        .insert(BackgroundColor(Color::WHITE));
}

pub fn update_loading_ui(
    event: On<ResourceLoaded>,
    mut loading_menu_progress: Single<&mut Node, With<LoadingMenuProgress>>,
) {
    let ResourceLoaded::InProgress { finished, waiting } = *event else {
        return;
    };

    let progress = finished as f32 / (finished + waiting) as f32 * 100.;
    loading_menu_progress.width = Val::Percent(progress);
}

pub fn despawn_loading_ui(loading_ui: Single<Entity, With<LoadingMenuUi>>, mut commands: Commands) {
    info!("Despawning Loading UI!");

    commands.entity(*loading_ui).despawn();
}
