use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    lighting::LineLight2d,
    player::PlayerHurtMarker,
    shared::GroupLabel,
    ui::tooltip::{TooltipDespawnSetting, TooltipSpawner},
};

use super::LevelSystems;

pub struct TooltipSignPlugin;

impl Plugin for TooltipSignPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<LdtkTooltipSign>("StaticTooltip")
            .add_systems(
                Update,
                (
                    display_tooltip_signs,
                    clean_tooltip_signs_on_reset.in_set(LevelSystems::Reset),
                ),
            );
    }
}

#[derive(Bundle)]
pub struct LdtkTooltipSign {
    collider: Collider,
    sensor: Sensor,
    collision_groups: CollisionGroups,
    sign: TooltipSign,
    sprite: Sprite,
    light: LineLight2d,
}

impl LdtkEntity for LdtkTooltipSign {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        _: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        asset_server: &AssetServer,
        _: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        Self {
            collider: Collider::cuboid(12., 12.),
            sensor: Sensor,
            collision_groups: CollisionGroups::new(
                GroupLabel::ALL,
                GroupLabel::PLAYER_COLLIDER | GroupLabel::PLAYER_SENSOR,
            ),
            sign: TooltipSign::from(entity_instance),
            sprite: Sprite::from_image(asset_server.load("tooltip-sign.png")),
            light: LineLight2d::point(Vec4::new(1., 1., 0., 1.), 40., 0.02),
        }
    }
}

pub fn clean_tooltip_signs_on_reset(mut commands: Commands, q_tooltip_signs: Query<&TooltipSign>) {
    for sign in q_tooltip_signs.iter() {
        if let Some(tooltip) = sign.tooltip {
            commands.entity(tooltip).despawn_recursive();
        }
    }
}

pub fn display_tooltip_signs(
    mut commands: Commands,
    mut tooltip_spawner: TooltipSpawner,
    rapier_context: Query<&RapierContext>,
    mut q_tooltip_signs: Query<(Entity, &mut TooltipSign)>,
    q_player: Query<Entity, With<PlayerHurtMarker>>,
) {
    let Ok(rapier) = rapier_context.get_single() else {
        return;
    };
    let Ok(player) = q_player.get_single() else {
        return;
    };

    for (sign_entity, mut sign) in q_tooltip_signs.iter_mut() {
        match (sign.tooltip, rapier.intersection_pair(sign_entity, player)) {
            (None, Some(true)) => {
                sign.tooltip = Some(tooltip_spawner.spawn_tooltip(
                    sign.text.clone(),
                    sign_entity,
                    Vec3::new(0., 25., 0.),
                    TooltipDespawnSetting::None,
                ));
            }
            (Some(tooltip), Some(false)) | (Some(tooltip), None) => {
                if let Some(cmds) = commands.get_entity(tooltip) {
                    cmds.despawn_recursive();
                };
                sign.tooltip = None;
            }
            _ => (),
        }
    }
}

#[derive(Component)]
pub struct TooltipSign {
    pub text: String,
    pub tooltip: Option<Entity>,
}

impl From<&EntityInstance> for TooltipSign {
    fn from(value: &EntityInstance) -> Self {
        let text = value
            .get_string_field("Text")
            .expect("Tooltip sign should have string field Text");

        Self {
            text: text.to_string(),
            tooltip: None,
        }
    }
}
