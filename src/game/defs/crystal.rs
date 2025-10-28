use std::collections::HashMap;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::tiles::TileTextureIndex;

use crate::{
    game::{
        defs::{
            merge_tile::{spawn_merged_tiles, MergedTile},
            DangerBox,
        },
        Layers, LevelSystems,
    },
    ldtk::LdtkLevelParam,
    shared::ResetLevels,
};
// use bevy_ecs_tilemap::tiles::TileTextureIndex;
// use bevy_rapier2d::prelude::*;
//
// use crate::{lighting::Occluder2d, shared::GroupLabel};
//
// use super::{
//     entity::HurtMarker,
//     merge_tile::{spawn_merged_tiles, MergedTile},
//     sensor::update_light_sensors,
//     CurrentLevel, LevelSystems,
// };

pub struct CrystalPlugin;

impl Plugin for CrystalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CrystalCache>().add_systems(
            PreUpdate,
            (
                invalidate_crystal_cache,
                init_crystal_cache_tiles,
                spawn_merged_tiles::<Crystal>,
                init_crystal_cache_groups,
            )
                .chain()
                .in_set(LevelSystems::Processing),
        );
        app.add_observer(on_crystal_changed);
        app.add_observer(reset_crystals);

        for i in 3..=10 {
            app.register_ldtk_int_cell_for_layer::<CrystalBundle>("Terrain", i);
        }

        for i in 1..=10 {
            app.register_ldtk_int_cell_for_layer::<CrystalIdBundle>("Crystalmap", i);
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrystalColor {
    #[default]
    Pink,
    Red,
    Blue,
    White,
}

impl CrystalColor {
    pub fn button_color(&self) -> Color {
        match self {
            CrystalColor::Pink => Color::srgb(1.5, 0.7, 1.0),
            CrystalColor::Red => Color::srgb(1.0, 0.0, 0.0),
            CrystalColor::White => Color::srgb(0.9, 0.9, 0.9),
            CrystalColor::Blue => Color::srgb(0.6, 1.1, 1.9),
        }
    }
}

impl From<&String> for CrystalColor {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "Pink" => CrystalColor::Pink,
            "Red" => CrystalColor::Red,
            "White" => CrystalColor::White,
            "Blue" => CrystalColor::Blue,
            _ => panic!("String does not represent a CrystalColor"),
        }
    }
}

/// Enum that represents the crystals that a [`LightSensor`] should toggle. Differs from the
/// LightColor in that the white color requires an ID field.
#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub struct CrystalIdent {
    pub color: CrystalColor,
    pub id: i32,
}

#[derive(Default, Component)]
pub struct Crystal {
    pub ident: CrystalIdent,
    init_active: bool,
    pub active: bool,
}

impl MergedTile for Crystal {
    type CompareData = (CrystalIdent, bool);

    fn bundle(
        commands: &mut EntityCommands,
        center: Vec2,
        extent: Vec2,
        compare_data: &Self::CompareData,
    ) {
        let (crystal_color, crystal_active) = compare_data;

        if crystal_color.color == CrystalColor::Blue {
            commands.insert(CollisionLayers::new(
                Layers::Terrain,
                [Layers::Terrain, Layers::LightSensor, Layers::CrystalShard],
            ));
        }

        dbg!(extent);
        if *crystal_active {
            commands.insert(
                Collider::rectangle(extent.x, extent.y),
                // Occluder2d::new(half_extent.x, half_extent.y),
            );
        }

        commands
            .insert(RigidBody::Static)
            .insert(Transform::from_xyz(center.x, center.y, 0.))
            .insert(CrystalGroup {
                representative: Crystal {
                    init_active: compare_data.1,
                    ident: compare_data.0,
                    active: compare_data.1,
                },
                extent,
            })
            .insert(DangerBox);
    }

    fn compare_data(&self) -> Self::CompareData {
        (self.ident, self.init_active)
    }
}

/// [`Bundle`] registered with [`LdktEntityAppExt::register_ldtk_entity`](LdtkEntityAppExt) to spawn
/// crystals directly from Ldtk.
#[derive(Bundle, LdtkIntCell)]
pub struct CrystalBundle {
    #[from_int_grid_cell]
    crystal: Crystal,
    #[from_int_grid_cell]
    cell: IntGridCell,
}

#[derive(Component)]
pub struct CrystalGroup {
    pub representative: Crystal,
    pub extent: Vec2,
}

/// Identifier [`Component`] used to label the ID of white crystals
#[derive(Default, Component, Clone, Copy, PartialEq)]
pub struct CrystalId(i32);

impl From<IntGridCell> for CrystalId {
    fn from(value: IntGridCell) -> Self {
        CrystalId(value.value)
    }
}

/// Bundle registered with LDTK to spawn in white crystal identifiers
#[derive(Default, Bundle, LdtkIntCell)]
pub struct CrystalIdBundle {
    #[from_int_grid_cell]
    id: CrystalId,
}

#[derive(Debug, Default, Resource)]
pub struct CrystalCache {
    tiles: HashMap<LevelIid, HashMap<CrystalIdent, Vec<Entity>>>,
    groups: HashMap<LevelIid, HashMap<CrystalIdent, Vec<Entity>>>,
}

fn invalidate_crystal_cache(
    mut ev_level: MessageReader<LevelEvent>,
    mut crystal_cache: ResMut<CrystalCache>,
) {
    for ev in ev_level.read() {
        let LevelEvent::Despawned(iid) = ev else {
            continue;
        };
        if let Some(mp) = crystal_cache.tiles.get_mut(iid) {
            mp.clear();
        }
        if let Some(mp) = crystal_cache.groups.get_mut(iid) {
            mp.clear();
        }
    }
}

fn init_crystal_cache_groups(
    q_crystal_groups: Query<(Entity, &ChildOf, &CrystalGroup), Added<CrystalGroup>>,
    q_level_iid: Query<&LevelIid>,
    mut crystal_cache: ResMut<CrystalCache>,
) {
    for (entity, ChildOf(parent), crystal_group) in q_crystal_groups.iter() {
        let Ok(level_iid) = q_level_iid.get(*parent) else {
            continue;
        };
        crystal_cache
            .groups
            .entry(level_iid.clone())
            .or_default()
            .entry(crystal_group.representative.ident)
            .or_default()
            .push(entity);
    }
}

/// System that will initialize all the crystals, storing their entities in the appropriate level
/// -> crystal color location in the crystal cache.
#[allow(clippy::type_complexity)]
fn init_crystal_cache_tiles(
    mut commands: Commands,
    q_crystal_id: Query<(&GridCoords, &ChildOf, &CrystalId), (Added<CrystalId>, Without<Crystal>)>,
    mut q_crystals: Query<(Entity, &GridCoords, &ChildOf, &mut Crystal), Added<Crystal>>,
    q_level_iid: Query<&LevelIid>,
    q_parent: Query<&ChildOf, (Without<CrystalId>, Without<Crystal>)>,
    mut crystal_cache: ResMut<CrystalCache>,
) {
    if q_crystals.is_empty() {
        return;
    }

    info!("Initializing crystals");

    // Hashmap of coordinates to color ids
    let mut coords_map: HashMap<LevelIid, HashMap<GridCoords, i32>> = HashMap::new();
    for (coords, ChildOf(parent), crystal_id) in q_crystal_id.iter() {
        let Ok(ChildOf(level_entity)) = q_parent.get(*parent) else {
            continue;
        };
        let Ok(level_iid) = q_level_iid.get(*level_entity) else {
            continue;
        };
        coords_map
            .entry(level_iid.clone())
            .or_default()
            .insert(*coords, crystal_id.0);

        commands.entity(*parent).insert(Visibility::Hidden);
    }

    for (entity, coord, ChildOf(parent), mut crystal) in q_crystals.iter_mut() {
        let Ok(ChildOf(level_entity)) = q_parent.get(*parent) else {
            continue;
        };
        let Ok(level_iid) = q_level_iid.get(*level_entity) else {
            continue;
        };

        // crystal.color is currently CrystalColor::White with id 0, we need to pull the proper ID
        // in if it exists
        let actual_color = CrystalIdent {
            color: crystal.ident.color,
            id: coords_map
                .get(level_iid)
                .and_then(|mp| mp.get(coord))
                .copied()
                .unwrap_or(0),
        };

        crystal_cache
            .tiles
            .entry(level_iid.clone())
            .or_default()
            .entry(actual_color)
            .or_default()
            .push(entity);

        crystal.ident = actual_color;
    }
}

fn is_crystal_active(cell_value: IntGridCell) -> bool {
    match cell_value.value {
        3 | 5 | 7 | 9 => true,
        4 | 6 | 8 | 10 => false,
        _ => panic!("Cell value does not correspond to crystal!"),
    }
}

fn crystal_color(cell_value: IntGridCell) -> CrystalColor {
    match cell_value.value {
        3 | 4 => CrystalColor::Pink,
        5 | 6 => CrystalColor::Red,
        7 | 8 => CrystalColor::White,
        9 | 10 => CrystalColor::Blue,
        _ => panic!("Cell value does not correspond to crystal!"),
    }
}

impl From<IntGridCell> for Crystal {
    fn from(cell: IntGridCell) -> Self {
        let init_active = is_crystal_active(cell);

        Crystal {
            ident: CrystalIdent {
                color: crystal_color(cell),
                // initialzed later in init_crystal_cache_tiles
                id: 0,
            },
            active: init_active,
            init_active,
        }
    }
}

/// The horizontal offset between active crystals and inactive crystals in the crystal tilemap
const CRYSTAL_INDEX_OFFSET: u32 = 5;

fn toggle_crystal_group(
    commands: &mut Commands,
    crystal_group_entity: Entity,
    crystal_group: &mut CrystalGroup,
) {
    let crystal = &mut crystal_group.representative;
    if !crystal.active {
        crystal.active = true;
        commands.entity(crystal_group_entity).insert(
            Collider::rectangle(crystal_group.extent.x, crystal_group.extent.y),
            // Occluder2d::new(crystal_group.half_extent.x, crystal_group.half_extent.y),
        );
    } else {
        crystal.active = false;
        commands.entity(crystal_group_entity).remove::<Collider>();
        // .remove::<Occluder2d>();
    }
}

fn toggle_crystal(crystal: &mut Crystal, crystal_index: &mut TileTextureIndex) {
    if !crystal.active {
        crystal.active = true;
        crystal_index.0 -= CRYSTAL_INDEX_OFFSET;
    } else {
        crystal.active = false;
        crystal_index.0 += CRYSTAL_INDEX_OFFSET;
    }
}

pub fn reset_crystals(
    _: On<ResetLevels>,
    mut commands: Commands,
    mut q_crystals: Query<(&mut Crystal, &mut TileTextureIndex)>,
    mut q_crystal_groups: Query<(Entity, &mut CrystalGroup)>,
) {
    for (entity, mut crystal_group) in q_crystal_groups.iter_mut() {
        let crystal = &crystal_group.representative;
        if crystal.init_active != crystal.active {
            toggle_crystal_group(&mut commands, entity, &mut crystal_group);
        }
    }

    for (mut crystal, mut index) in q_crystals.iter_mut() {
        if crystal.init_active != crystal.active {
            toggle_crystal(&mut crystal, &mut index);
        }
    }
}

/// Event that will toggle all crystals of a certain color.
#[derive(Event)]
pub struct CrystalToggleEvent {
    pub ident: CrystalIdent,
}

/// [`System`] that listens to when [`Crystal`]s are activated or deactivated, updating the
/// [`Sprite`] and adding/removing [`FixedEntityBundle`] of the [`Entity`].
pub fn on_crystal_changed(
    event: On<CrystalToggleEvent>,
    mut commands: Commands,
    mut q_crystal: Query<(&mut Crystal, &mut TileTextureIndex)>,
    mut q_crystal_groups: Query<&mut CrystalGroup>,
    crystal_cache: Res<CrystalCache>,
    ldtk_level_param: LdtkLevelParam,
) {
    let iid = ldtk_level_param.cur_iid().expect("Cur level should exist");

    let Some(crystal_tile_map) = crystal_cache.tiles.get(&iid) else {
        return;
    };
    let Some(crystal_group_map) = crystal_cache.groups.get(&iid) else {
        return;
    };

    if let Some(crystals) = crystal_tile_map.get(&event.ident) {
        for crystal_entity in crystals.iter() {
            let Ok((mut crystal, mut index)) = q_crystal.get_mut(*crystal_entity) else {
                continue;
            };
            toggle_crystal(&mut crystal, &mut index);
        }
    };
    if let Some(crystal_groups) = crystal_group_map.get(&event.ident) {
        for crystal_group_entity in crystal_groups.iter() {
            let Ok(mut crystal_group) = q_crystal_groups.get_mut(*crystal_group_entity) else {
                continue;
            };
            toggle_crystal_group(&mut commands, *crystal_group_entity, &mut crystal_group);
        }
    }
}
