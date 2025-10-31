use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{shared::ResetLevels, ui::UiFont};

use super::follow::WorldSpacePositionTarget;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_tooltip_despawns);
    }
}

#[derive(Component)]
pub struct Tooltip;

pub fn handle_tooltip_despawns(
    _: On<ResetLevels>,
    mut commands: Commands,
    q_tooltips: Query<Entity, With<Tooltip>>,
) {
    for tooltip in q_tooltips.iter() {
        commands.entity(tooltip).despawn();
    }
}

#[derive(SystemParam)]
pub struct TooltipSpawner<'w, 's> {
    commands: Commands<'w, 's>,
    ui_font: Res<'w, UiFont>,
}

impl TooltipSpawner<'_, '_> {
    pub fn spawn_tooltip(
        &mut self,
        tooltip: impl Into<String>,
        on_entity: Entity,
        offset: Vec3,
    ) -> Entity {
        let container = self
            .commands
            .spawn(Node {
                max_width: Val::Vw(20.),
                padding: UiRect::all(Val::Px(12.)),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(2.0)),

                ..default()
            })
            .insert(UiTransform {
                translation: Val2::percent(-50., 0.),
                ..default()
            })
            .insert(GlobalZIndex(-1))
            .insert(BackgroundColor(Color::srgba(0., 0., 0., 0.5)))
            .insert(BorderColor::all(Color::srgba(1., 1., 1., 0.7)))
            .insert(Tooltip)
            .insert(WorldSpacePositionTarget {
                target: on_entity,
                offset,
            })
            .id();

        self.commands
            .spawn(Text::new(tooltip.into()))
            .insert(self.ui_font.text_font())
            .insert(ChildOf(container))
            .insert(TextColor(Color::srgba(1., 1., 1., 0.7)))
            .insert(TextLayout::new_with_justify(Justify::Center));

        container
    }
}
