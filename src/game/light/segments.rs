use avian2d::prelude::*;
use bevy::{ecs::entity::EntityHashMap, prelude::*};

use crate::{
    camera::HIGHRES_LAYER,
    game::{
        light::{
            render::{LightMaterial, LightRenderData},
            HitByLight, LightBeamSource, LightColor, LIGHT_SPEED,
        },
        lighting::LineLight2d,
        Layers,
    },
    shared::ResetLevels,
};

/// Marker [`Component`] used to query for light segments.
#[derive(Default, Component, Clone, Debug)]
pub struct LightSegment {
    pub color: LightColor,
}

/// [`Bundle`] used in the initialization of the [`LightSegmentCache`] to spawn segment entities.
#[derive(Bundle, Debug, Clone, Default)]
pub struct LightSegmentBundle {
    pub segment: LightSegment,
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<LightMaterial>,
    pub visibility: Visibility,
    pub transform: Transform,
}

/// [`Resource`] used to store [`Entity`] handles to the light segments so they aren't added and
/// despawned every frame. See [`simulate_light_sources`] for details.
#[derive(Resource, Default)]
pub struct LightSegmentCache {
    segments: EntityHashMap<(Vec<Entity>, LightColor)>,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LightBounceSfx {
    #[dependency]
    bounce: [Handle<AudioSource>; 3],
    #[dependency]
    reflect: [Handle<AudioSource>; 3],
}

impl FromWorld for LightBounceSfx {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        LightBounceSfx {
            bounce: [
                asset_server.load("sfx/light/light-bounce-1.wav"),
                asset_server.load("sfx/light/light-bounce-2.wav"),
                asset_server.load("sfx/light/light-bounce-3.wav"),
            ],
            reflect: [
                asset_server.load("sfx/light/light-bounce-1-reflect.wav"),
                asset_server.load("sfx/light/light-bounce-2-reflect.wav"),
                asset_server.load("sfx/light/light-bounce-3-reflect.wav"),
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LightBeamIntersection {
    pub entity: Entity,
    pub point: Vec2,
    pub time: f32,
}

/// Stores information about the trajectory of a LightBeam
#[derive(Debug)]
pub struct LightBeamPlayback {
    pub intersections: Vec<LightBeamIntersection>,
    pub end_point: Option<Vec2>,
    pub elapsed_time: f32,
}

impl LightBeamPlayback {
    pub fn iter_points<'a>(
        &'a self,
        source: &'a LightBeamSource,
    ) -> impl Iterator<Item = Vec2> + 'a {
        std::iter::once(source.start_pos)
            .chain(
                self.intersections
                    .iter()
                    .map(|intersection| intersection.point),
            )
            .chain(self.end_point.iter().copied())
    }
}

#[derive(Default, Debug, Component)]
pub struct PrevLightBeamPlayback {
    pub intersections: Vec<Option<LightBeamIntersection>>,
}

const LIGHT_MAX_SEGMENTS: usize = 15;

pub fn play_light_beam(
    spatial_query: &SpatialQuery,
    source: &LightBeamSource,
    // black_ray_qry: &Query<(Entity, &BlackRayComponent)>,
    // q_mirrors: &Query<&Mirror>,
) -> LightBeamPlayback {
    let mut ray_pos = source.start_pos;
    let mut ray_dir = source.start_dir;
    let collision_groups = match source.color {
        LightColor::White => {
            CollisionLayers::new(Layers::WhiteRay, [Layers::Terrain, Layers::LightSensor])
        }
        // LightColor::Black => {
        //     CollisionLayers::new(Layers::BlackRay, Layers::Terrain, Layers::LightSensor)
        // }
        LightColor::Blue => CollisionLayers::new(
            Layers::BlueRay,
            [Layers::Terrain, Layers::LightSensor, Layers::WhiteRay],
        ),
        _ => CollisionLayers::new(
            Layers::LightRay,
            [Layers::Terrain, Layers::LightSensor, Layers::WhiteRay],
        ),
    };

    let mut ray_qry = SpatialQueryFilter::default().with_mask(collision_groups.filters);
    let mut remaining_time = source.time_traveled;

    let mut playback = LightBeamPlayback {
        intersections: vec![],
        end_point: None,
        elapsed_time: 0.0,
    };

    // for _ in 0..source.color.num_bounces() + 1 {
    let num_segments = source.color.num_bounces() + 1;

    let mut i = 0;
    // let mut extra_bounces_from_mirror = 0;
    // while i < num_segments + extra_bounces_from_mirror && i < LIGHT_MAX_SEGMENTS {
    while i < num_segments && i < LIGHT_MAX_SEGMENTS {
        let Some(hit) = spatial_query.cast_ray(ray_pos, ray_dir, remaining_time, true, &ray_qry)
        else {
            let final_point = ray_pos + ray_dir * remaining_time;
            playback.elapsed_time += remaining_time;
            playback.end_point = Some(final_point);
            break;
        };
        // if q_mirrors.contains(hit.entity) {
        //     extra_bounces_from_mirror += 1;
        // }

        // if inside something???
        let mut ignore_entity = true;
        if hit.distance < 0.01 {
            ignore_entity = false;
        }

        playback.elapsed_time += hit.distance;
        remaining_time -= hit.distance;
        let hit_point = ray_pos + *ray_dir * hit.distance;

        playback.intersections.push(LightBeamIntersection {
            entity: hit.entity,
            point: hit_point,
            time: playback.elapsed_time,
        });

        ray_pos = hit_point;
        ray_dir =
            Dir2::new((Vec2::from(ray_dir)).reflect(hit.normal)).expect("cast dir cannot be 0");
        if ignore_entity {
            ray_qry = ray_qry.with_excluded_entities([hit.entity]);
        }

        // if black_ray_qry.get(hit.entity).is_ok() {
        //     break;
        // }
        i += 1;
    }

    playback
}

#[derive(Default, Component)]
pub struct LightBeamPoints(Vec<Vec2>);

/// [`System`] that runs on [`Update`], calculating the [`Transform`] of light segments from the
/// corresponding [`LightBeamSource`]. Note that this calculation happens every frame, so instead of
/// rapidly spawning/despawning the entities, we spawn them and cache them in the
/// [`LightSegmentCache`], then modify their [`Visibility`] and [`Transform`]s.
///
/// If needed, optimization work can be done by recalculating only segments that are currently
/// changing (segments already "stabilized" usually won't move).
#[allow(clippy::too_many_arguments)]
pub fn simulate_light_sources(
    mut commands: Commands,
    mut q_light_sources: Query<(Entity, &mut LightBeamSource, &mut PrevLightBeamPlayback)>,
    // q_black_ray: Query<(Entity, &BlackRayComponent)>,
    spatial_query: SpatialQuery,
    // q_mirrors: Query<&Mirror>,
    // used to tell if a collision was against a white beam (a different sound is played)
    q_segments: Query<&LightSegment>,
    light_bounce_sfx: Res<LightBounceSfx>,
    // mut ev_spark_explosion: EventWriter<SparkExplosionEvent>,
) {
    for (source_entity, mut source, mut prev_playback) in q_light_sources.iter_mut() {
        let playback = play_light_beam(&spatial_query, &source);
        let mut pts: Vec<Vec2> = playback.iter_points(&source).collect();

        let intersections = playback.intersections.len();
        for i in 0..intersections {
            let prev_x = prev_playback.intersections.get(i).cloned().flatten();
            let new_x = playback.intersections[i];

            let is_same_intersection = prev_x.is_some_and(|prev_x| prev_x.entity == new_x.entity);

            // diff intersection
            if !is_same_intersection {
                let is_closer = prev_x.is_none_or(|prev_x| prev_x.time > new_x.time);

                // remvoe all points after the current intersection
                pts.truncate(i + 2);

                let add_intersection = prev_x.is_none() || is_closer;
                let remove_intersection = prev_x.is_some();
                let play_sound = prev_x.is_none() && !is_same_intersection;

                // handle remove before add because it could be the case that both are true
                if remove_intersection {
                    pts[i + 1] = prev_x.unwrap().point;
                    commands.trigger(HitByLight {
                        entity: prev_x.unwrap().entity,
                        color: source.color,
                        hit: false,
                    });
                    prev_playback.intersections[i] = None;
                    source.time_traveled = prev_x.unwrap().time;

                    // unhit all future lights as well
                    for j in i + 1..prev_playback.intersections.len() {
                        let Some(intersection) = prev_playback.intersections[j] else {
                            continue;
                        };
                        commands.trigger(HitByLight {
                            entity: intersection.entity,
                            color: source.color,
                            hit: false,
                        });
                    }
                }

                if add_intersection {
                    pts[i + 1] = new_x.point;
                    commands.trigger(HitByLight {
                        entity: new_x.entity,
                        color: source.color,
                        hit: true,
                    });
                    if i >= prev_playback.intersections.len() {
                        assert!(i == prev_playback.intersections.len());
                        prev_playback.intersections.push(Some(new_x));
                    } else {
                        prev_playback.intersections[i] = Some(new_x);
                    }
                    source.time_traveled = new_x.time;
                }

                if play_sound {
                    let reflect = match q_segments.get(new_x.entity) {
                        Ok(segment) => segment.color == LightColor::White,
                        _ => false,
                    };
                    let audio = if reflect {
                        light_bounce_sfx
                            .reflect
                            .get(i)
                            .unwrap_or(&light_bounce_sfx.reflect[2])
                    } else {
                        light_bounce_sfx
                            .bounce
                            .get(i)
                            .unwrap_or(&light_bounce_sfx.bounce[2])
                    }
                    .clone();
                    // ev_spark_explosion.send(SparkExplosionEvent {
                    //     pos: new_x.point,
                    //     color: source.color.light_beam_color(),
                    // });
                    commands
                        .entity(new_x.entity)
                        .with_child((AudioPlayer::new(audio), PlaybackSettings::DESPAWN));
                }

                prev_playback.intersections.truncate(i + 1);
                break;
            } else {
                // keep on updating the previous intersection buffer because this could be a moving
                // platform
                prev_playback.intersections[i] = Some(new_x);
            }
        }
        commands.entity(source_entity).insert(LightBeamPoints(pts));
    }
}

pub fn spawn_needed_segments(
    mut commands: Commands,
    q_light_sources: Query<(Entity, &LightBeamSource, &LightBeamPoints)>,
    mut segment_cache: ResMut<LightSegmentCache>,
    light_render_data: Res<LightRenderData>,
) {
    for (entity, source, pts) in q_light_sources.iter() {
        let segments = pts.0.len() - 1;
        // lazily spawn segment entities until there are enough segments to display the light beam
        // path
        if !segment_cache.segments.contains_key(&entity) {
            segment_cache
                .segments
                .insert(entity, (vec![], source.color));
        }

        while segment_cache.segments[&entity].0.len() < segments.min(LIGHT_MAX_SEGMENTS) {
            let id = commands
                .spawn(LightSegmentBundle {
                    segment: LightSegment {
                        color: source.color,
                    },
                    mesh: light_render_data.mesh.clone(),
                    material: light_render_data.material_map[source.color].clone(),
                    visibility: Visibility::Hidden,
                    transform: Transform::default(),
                })
                .insert(HIGHRES_LAYER)
                .with_child(LineLight2d {
                    color: source.color.lighting_color().extend(1.0),
                    half_length: 10.0,
                    radius: 20.0,
                    volumetric_intensity: 0.04,
                })
                .id();
            // White beams need colliders
            if source.color == LightColor::White {
                commands.entity(id).insert((
                    Collider::rectangle(1., 1.),
                    Sensor,
                    CollisionLayers::new(
                        Layers::WhiteRay,
                        [
                            Layers::Terrain,
                            Layers::LightSensor,
                            Layers::LightRay,
                            Layers::BlueRay,
                        ],
                    ),
                ));
            }
            // // Black beams need Black_Ray_Component and colliders
            // if source.color == LightColor::Black {
            //     commands.entity(id).insert((
            //         BlackRayComponent,
            //         Sensor,
            //         Collider::cuboid(0.5, 0.5),
            //         CollisionGroups::new(
            //             GroupLabel::BLACK_RAY,
            //             GroupLabel::TERRAIN
            //                 | GroupLabel::PLATFORM
            //                 | GroupLabel::LIGHT_SENSOR
            //                 | GroupLabel::LIGHT_RAY
            //                 | GroupLabel::BLUE_RAY
            //                 | GroupLabel::WHITE_RAY,
            //         ),
            //     ));
            // }
            segment_cache.segments.get_mut(&entity).unwrap().0.push(id);
            //segment_cache.segments[&source.color].push(id);
        }
    }
}

pub fn visually_sync_segments(
    q_light_sources: Query<(Entity, &LightBeamSource, &LightBeamPoints)>,
    segment_cache: Res<LightSegmentCache>,
    mut q_segments: Query<(&Children, &mut Transform, &mut Visibility), With<LightSegment>>,
    mut q_line_lights: Query<&mut LineLight2d>,
) {
    for (entity, _source, pts) in q_light_sources.iter() {
        let pts = &pts.0;
        // use the light beam path to set the transform of the segments currently in the cache

        for (i, segment) in segment_cache.segments[&entity].0.iter().enumerate() {
            let Ok((children, mut c_transform, mut c_visibility)) = q_segments.get_mut(*segment)
            else {
                panic!("Segment doesn't have transform or visibility!");
            };
            let Some(line_light_entity) = children.first() else {
                panic!("Segment doesn't have line light!");
            };
            let Ok(mut line_light) = q_line_lights.get_mut(*line_light_entity) else {
                panic!("Segment doesn't have line light!");
            };

            if i + 1 < pts.len() && pts[i].distance(pts[i + 1]) > 0.1 {
                // NOTE: hardcode here should be okay
                let midpoint = pts[i].midpoint(pts[i + 1]).extend(4.);
                let scale = Vec3::new(pts[i].distance(pts[i + 1]), 1., 1.);
                let rotation = (pts[i + 1] - pts[i]).to_angle();

                let transform = Transform::from_translation(midpoint)
                    .with_scale(scale)
                    .with_rotation(Quat::from_rotation_z(rotation));

                line_light.half_length = scale.x / 2.0;
                *c_transform = transform;
                *c_visibility = Visibility::Visible;
            } else {
                // required for white beam
                line_light.half_length = 0.0;
                *c_transform = Transform::default();
                *c_visibility = Visibility::Hidden;
            }
        }
    }
}

/// [`System`] that runs on [`FixedUpdate`], advancing the distance the light beam can travel.
pub fn tick_light_sources(mut q_light_sources: Query<&mut LightBeamSource>) {
    for mut source in q_light_sources.iter_mut() {
        source.time_traveled += LIGHT_SPEED;
    }
}

/// [`System`] that is responsible for hiding all of the [`LightSegment`](LightSegmentBundle)s
/// and despawning [`LightBeamSource`]s when the level changes.
pub fn cleanup_light_sources(
    _: On<ResetLevels>,
    mut commands: Commands,
    q_light_sources: Query<(Entity, &LightBeamSource)>,
    segment_cache: Res<LightSegmentCache>,
    mut q_segments: Query<(&mut Transform, &mut Visibility), With<LightSegment>>,
) {
    for (entity, _) in q_light_sources.iter() {
        // if light_beam_source.color != LightColor::Black {
        commands.entity(entity).despawn();
        // }
    }

    segment_cache.segments.iter().for_each(|(_, items)| {
        for &entity in items.0.iter() {
            let (mut transform, mut visibility) = q_segments
                .get_mut(entity)
                .expect("Segment should have visibility");

            // required for white beam
            *transform = Transform::default();
            *visibility = Visibility::Hidden;
        }
    });
}
