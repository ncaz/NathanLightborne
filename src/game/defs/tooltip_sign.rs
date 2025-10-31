use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    asset::LoadResource,
    game::{lighting::LineLight2d, Layers},
    ui::tooltip::TooltipSpawner,
};

pub struct TooltipSignPlugin;

impl Plugin for TooltipSignPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TooltipSignAssets>();
        app.load_resource::<TooltipSignAssets>();
        app.register_ldtk_entity::<LdtkTooltipSign>("StaticTooltip");
        app.add_observer(on_add_tooltip_sign);
    }
}

#[derive(Resource, Asset, Reflect, Clone)]
#[reflect(Resource)]
pub struct TooltipSignAssets {
    #[dependency]
    sign: Handle<Image>,
}

impl FromWorld for TooltipSignAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            sign: asset_server.load("tooltip-sign.png"),
        }
    }
}

#[derive(Bundle, LdtkEntity)]
pub struct LdtkTooltipSign {
    #[from_entity_instance]
    sign: TooltipSign,
}

pub fn on_add_tooltip_sign(
    event: On<Add, TooltipSign>,
    mut commands: Commands,
    tooltip_sign_assets: Res<TooltipSignAssets>,
) {
    commands
        .entity(event.entity)
        .insert(Collider::rectangle(24., 24.))
        .insert(Sensor)
        .insert(CollisionLayers::new(
            Layers::SensorBox,
            Layers::PlayerHurtbox,
        ))
        .insert(Sprite::from_image(tooltip_sign_assets.sign.clone()))
        .insert(LineLight2d::point(Vec4::new(1., 1., 0., 1.), 40., 0.02));
}

pub fn hide_tooltip_signs(
    event: On<CollisionEnd>,
    mut commands: Commands,
    mut q_tooltip_signs: Query<&mut TooltipSign>,
) {
    let Ok(mut sign) = q_tooltip_signs.get_mut(event.collider2) else {
        return;
    };
    let Some(tooltip) = sign.tooltip else {
        return;
    };
    if let Ok(mut cmds) = commands.get_entity(tooltip) {
        cmds.despawn();
    };
    sign.tooltip = None;
}

pub fn display_tooltip_signs(
    event: On<CollisionStart>,
    mut tooltip_spawner: TooltipSpawner,
    mut q_tooltip_signs: Query<&mut TooltipSign>,
) {
    let Ok(mut sign) = q_tooltip_signs.get_mut(event.collider2) else {
        return;
    };
    sign.tooltip = Some(tooltip_spawner.spawn_tooltip(
        sign.text.clone(),
        event.collider2,
        Vec3::new(0., 15., 0.),
    ));
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
