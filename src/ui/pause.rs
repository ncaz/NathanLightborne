use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    asset::LoadResource,
    shared::{GameState, PlayState, UiState},
    ui::{UiButton, UiClick, UiFont, UiFontSize},
};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PauseAssets>();
        app.load_resource::<PauseAssets>();
        app.add_systems(OnEnter(PlayState::Paused), spawn_pause);
        app.add_systems(OnExit(PlayState::Paused), despawn_pause);
        app.add_systems(
            Update,
            toggle_pause
                .run_if(in_state(GameState::InGame))
                .run_if(input_just_pressed(KeyCode::Escape)),
        );
    }
}

#[derive(Resource, Asset, Reflect, Clone)]
#[reflect(Resource)]
pub struct PauseAssets {
    #[dependency]
    bg: Handle<Image>,
}

impl FromWorld for PauseAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            bg: asset_server.load("ui/pause_menu.png"),
        }
    }
}

#[derive(Component)]
pub struct PauseMarker;

fn spawn_pause(mut commands: Commands, ui_font: Res<UiFont>, pause_assets: Res<PauseAssets>) {
    let container = commands
        .spawn(PauseMarker)
        .insert(Node {
            width: Val::Vw(60.),
            height: Val::Vh(60.),
            margin: UiRect::new(Val::Vw(20.), Val::Vw(20.), Val::Vh(20.), Val::Vh(20.)),
            justify_content: JustifyContent::SpaceBetween,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(96.0)),
            column_gap: Val::Px(32.),
            row_gap: Val::Px(32.),
            ..default()
        })
        .insert(ImageNode::from(pause_assets.bg.clone()).with_mode(NodeImageMode::Stretch))
        // .insert(BackgroundColor(Color::BLACK.with_alpha(0.2)))
        .id();

    commands
        .spawn(Text::new("Paused"))
        .insert(ui_font.text_font().with_font_size(UiFontSize::HEADER))
        .insert(ChildOf(container));

    let center_container = commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(32.),
            row_gap: Val::Px(32.),
            ..default()
        })
        .insert(ChildOf(container))
        .id();

    commands
        .spawn(Text::new("Main Menu"))
        .insert(Button)
        .insert(UiButton)
        .insert(ChildOf(center_container))
        .insert(ui_font.text_font().with_font_size(UiFontSize::BUTTON))
        .observe(
            |_: On<UiClick>,
             mut next_game_state: ResMut<NextState<GameState>>,
             mut next_ui_state: ResMut<NextState<UiState>>| {
                next_game_state.set(GameState::Ui);
                next_ui_state.set(UiState::StartMenu);
            },
        );

    commands
        .spawn(Text::new("Resume"))
        .insert(Button)
        .insert(UiButton)
        .insert(ui_font.text_font().with_font_size(UiFontSize::BUTTON))
        .insert(ChildOf(center_container))
        .observe(
            |_: On<UiClick>, mut next_play_state: ResMut<NextState<PlayState>>| {
                next_play_state.set(PlayState::Playing);
            },
        );
}

fn despawn_pause(mut commands: Commands, pause: Single<Entity, With<PauseMarker>>) {
    commands.entity(*pause).despawn();
}

fn toggle_pause(state: Res<State<PlayState>>, mut next_state: ResMut<NextState<PlayState>>) {
    match state.get() {
        PlayState::Paused => next_state.set(PlayState::Playing),
        PlayState::Playing => next_state.set(PlayState::Paused),
        _ => {}
    }
}
