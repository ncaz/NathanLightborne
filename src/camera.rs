use std::time::Duration;

use bevy::{
    camera::{visibility::RenderLayers, RenderTarget},
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::Hdr,
    },
};

use crate::{callback::Callback, game::lighting::AmbientLight2d};

pub const CAMERA_WIDTH: u32 = 320;
pub const CAMERA_HEIGHT: u32 = 180;

pub const TERRAIN_LAYER: RenderLayers = RenderLayers::layer(0);
pub const LYRA_LAYER: RenderLayers = RenderLayers::layer(1);
pub const HIGHRES_LAYER: RenderLayers = RenderLayers::layer(2);
pub const TRANSITION_LAYER: RenderLayers = RenderLayers::layer(5);

/// The [`Plugin`] responsible for handling anything Camera related.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraAnimations>();
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, animate_camera_zoom);
        app.add_systems(Update, animate_camera_move);
        app.add_observer(handle_zoom_camera);
        app.add_observer(handle_move_camera);
        app.add_systems(
            PostUpdate,
            apply_camera_snapping.before(TransformSystems::Propagate),
        );
    }
}

#[derive(Component, Default)]
pub struct MainCamera;

// #[derive(Component)]
// pub struct TransitionCamera;
//
// #[derive(Component)]
// pub struct TransitionMeshMarker;

#[derive(Component)]
#[require(Transform)]
pub struct PixelPerfectCamera {
    pub snap_entity: Entity,
}

pub fn apply_camera_snapping(
    q_camera: Query<&Transform, With<MainCamera>>,
    mut q_transform: ParamSet<(TransformHelper, Query<&mut Transform, Without<MainCamera>>)>,
    q_match_camera: Query<(Entity, &PixelPerfectCamera)>,
) {
    let Ok(main_camera_transform) = q_camera.single() else {
        return;
    };
    for (camera_entity, pixel_offset) in q_match_camera.iter() {
        let Ok(snap_transform) = q_transform
            .p0()
            .compute_global_transform(pixel_offset.snap_entity)
        else {
            continue;
        };

        let snap_to_vec =
            main_camera_transform.translation - snap_transform.compute_transform().translation;
        let move_vec = (snap_to_vec.round() - snap_to_vec).xy();

        let mut helper = q_transform.p1();
        let Ok(mut transform) = helper.get_mut(camera_entity) else {
            continue;
        };
        transform.translation = move_vec.extend(transform.translation.z);
    }
}

pub fn build_render_target(width: u32, height: u32) -> (Image, Projection) {
    let canvas_size = Extent3d {
        width,
        height,
        ..default()
    };

    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    canvas.resize(canvas_size);

    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::Fixed {
            width: width as f32,
            height: height as f32,
        },
        ..OrthographicProjection::default_2d()
    });

    (canvas, projection)
}

// pub fn setup_transition_camera(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     let projection = OrthographicProjection {
//         scaling_mode: bevy::camera::ScalingMode::Fixed {
//             width: CAMERA_WIDTH as f32,
//             height: CAMERA_HEIGHT as f32,
//         },
//         ..OrthographicProjection::default_2d()
//     };
//
//     commands.spawn((
//         Camera2d,
//         TransitionCamera,
//         Camera {
//             order: 3,
//             clear_color: ClearColorConfig::None,
//             ..default()
//         },
//         projection.clone(),
//         Transform::default(),
//         TRANSITION_LAYER,
//     ));
//
//     commands.spawn((
//         Mesh2d(meshes.add(Rectangle::new(CAMERA_WIDTH as f32, CAMERA_HEIGHT as f32))),
//         MeshMaterial2d(materials.add(Color::BLACK)),
//         // send to narnia, should be moved by any animations using it
//         Transform::from_xyz(10000.0, 10000.0, 0.0),
//         TransitionMeshMarker,
//         TRANSITION_LAYER,
//     ));
// }
//
/// [`Startup`] [`System`] that spawns the [`Camera2d`] in the world.
///
/// Notes:
/// - Spawns the camera with [`OrthographicProjection`] with fixed scaling at 320x180
pub fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::Fixed {
            width: CAMERA_WIDTH as f32,
            height: CAMERA_HEIGHT as f32,
        },
        ..OrthographicProjection::default_2d()
    });

    let main_camera = commands
        .spawn(Camera2d)
        .insert(MainCamera)
        .insert(Camera {
            order: 2, // must be after the lowres layers
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        })
        .insert(Hdr)
        .insert(Tonemapping::TonyMcMapface)
        .insert(projection)
        .insert(Transform::default())
        .insert(HIGHRES_LAYER)
        .id();

    let (terrain_image, terrain_projection) =
        build_render_target(CAMERA_WIDTH + 2, CAMERA_HEIGHT + 2);
    let terrain_handle = images.add(terrain_image);

    // spawn a dummy entity so that cameras can snap to 0
    let origin_entity = commands.spawn(Transform::default()).id();

    commands
        .spawn(Camera2d)
        .insert(Camera {
            order: 0,
            target: RenderTarget::Image(terrain_handle.clone().into()),
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        })
        .insert(Hdr)
        .insert(AmbientLight2d {
            color: Vec4::new(1.0, 1.0, 1.0, 0.4),
        })
        .insert(Tonemapping::TonyMcMapface)
        .insert(PixelPerfectCamera {
            snap_entity: origin_entity,
        })
        .insert(terrain_projection)
        .insert(Transform::default())
        .insert(TERRAIN_LAYER)
        .insert(ChildOf(main_camera))
        .with_child((Sprite::from_image(terrain_handle.clone()), HIGHRES_LAYER));
}

// #[derive(Debug)]
// pub enum CameraTransition {
//     SlideToBlack,
//     SlideFromBlack,
// }
//
// #[derive(Event, Debug)]
// pub struct CameraTransitionEvent {
//     pub duration: Duration,
//     pub ease_fn: EaseFunction,
//     pub callback: Option<SystemId>,
//     pub effect: CameraTransition,
// }

#[derive(Debug)]
pub enum CameraControlType {
    Animated {
        duration: Duration,
        ease_fn: EaseFunction,
        callback_entity: Option<Entity>,
    },
    Instant,
}

#[derive(Event, Debug)]
pub struct CameraMoveEvent {
    pub to: Vec2,
    pub variant: CameraControlType,
}

#[derive(Event, Debug)]
pub struct CameraZoomEvent {
    pub scale: f32,
    pub variant: CameraControlType,
}

#[derive(Debug)]
pub struct CameraAnimationInfo<LERP: Sized> {
    progress: Timer,
    start: LERP,
    end: LERP,
    curve: EasingCurve<f32>,
    callback_entity: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct CameraAnimations {
    translation: Option<CameraAnimationInfo<Vec3>>,
    zoom: Option<CameraAnimationInfo<f32>>,
}

// pub fn animate_transition(
//     mut commands: Commands,
//     mut transition_mesh: Single<&mut Transform, With<TransitionMeshMarker>>,
//     mut animation: Local<Option<CameraAnimationInfo<Vec3>>>,
//     time: Res<Time>,
// ) {
//     let Some(mut anim) = *animation else {
//         return;
//     };
//     anim.progress.tick(time.delta());
//     let percent = anim.progress.elapsed_secs() / anim.progress.duration().as_secs_f32();
//     mesh_transform.translation = anim
//         .start
//         .lerp(anim.end, anim.curve.sample_clamped(percent));
//
//     if anim.progress.just_finished() {
//         if anim.callback.is_some() {
//             commands.run_system(anim.callback.unwrap());
//         }
//         *animation = None;
//     }
// }
//
// pub fn on_transition(
//     event: On<CameraTransitionEvent>,
//     mut commands: Commands,
//     mut q_transition_mesh: Query<&mut Transform, With<TransitionMeshMarker>>,
//     mut animation: Local<Option<CameraAnimationInfo<Vec3>>>,
//     time: Res<Time>,
// ) {
//     let Ok(mut mesh_transform) = q_transition_mesh.single_mut() else {
//         return;
//     };
//
//     let mut anim = match event.effect {
//         CameraTransition::SlideFromBlack => CameraAnimationInfo {
//             progress: Timer::new(event.duration, TimerMode::Once),
//             start: Vec3::new(0.0, 0.0, 0.0),
//             end: Vec3::new(0.0, -(CAMERA_HEIGHT as f32), 0.0),
//             curve: EasingCurve::new(0.0, 1.0, event.ease_fn),
//             callback: event.callback,
//         },
//         CameraTransition::SlideToBlack => CameraAnimationInfo {
//             progress: Timer::new(event.duration, TimerMode::Once),
//             start: Vec3::new(0.0, CAMERA_HEIGHT as f32, 0.0),
//             end: Vec3::new(0.0, 0.0, 0.0),
//             curve: EasingCurve::new(0.0, 1.0, event.ease_fn),
//             callback: event.callback,
//         },
//     };
//     *animation = Some(anim);
//
//     let Some(mut anim) = *animation else {
//         return;
//     };
//     anim.progress.tick(time.delta());
//     let percent = anim.progress.elapsed_secs() / anim.progress.duration().as_secs_f32();
//     mesh_transform.translation = anim
//         .start
//         .lerp(anim.end, anim.curve.sample_clamped(percent));
//
//     if anim.progress.just_finished() {
//         if anim.callback.is_some() {
//             commands.run_system(anim.callback.unwrap());
//         }
//         *animation = None;
//     }
// }

pub fn handle_move_camera(
    event: On<CameraMoveEvent>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
    mut animations: ResMut<CameraAnimations>,
) {
    match event.variant {
        CameraControlType::Animated {
            duration,
            ease_fn,
            callback_entity,
        } => {
            let anim = CameraAnimationInfo {
                progress: Timer::new(duration, TimerMode::Once),
                start: camera.translation,
                end: event.to.extend(camera.translation.z),
                curve: EasingCurve::new(0.0, 1.0, ease_fn),
                callback_entity,
            };
            animations.translation = Some(anim);
        }
        CameraControlType::Instant => {
            camera.translation = event.to.extend(camera.translation.z);
        }
    }
}

pub fn animate_camera_move(
    mut commands: Commands,
    mut animations: ResMut<CameraAnimations>,
    time: Res<Time>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
) {
    let Some(ref mut anim) = animations.translation else {
        return;
    };

    anim.progress.tick(time.delta());

    let percent = anim.progress.elapsed_secs() / anim.progress.duration().as_secs_f32();
    camera.translation = anim
        .start
        .lerp(anim.end, anim.curve.sample_clamped(percent));

    if anim.progress.just_finished() {
        if let Some(callback_entity) = anim.callback_entity {
            commands.trigger(Callback {
                entity: callback_entity,
            });
        }
        animations.translation = None;
    }
}

pub fn handle_zoom_camera(
    event: On<CameraZoomEvent>,
    mut camera: Single<&mut Projection, With<MainCamera>>,
    mut animations: ResMut<CameraAnimations>,
) {
    let Projection::Orthographic(ref mut projection) = **camera else {
        panic!("Camera can only have orthographic projection");
    };

    match event.variant {
        CameraControlType::Animated {
            duration,
            ease_fn,
            callback_entity,
        } => {
            let anim = CameraAnimationInfo {
                progress: Timer::new(duration, TimerMode::Once),
                start: projection.scale,
                end: event.scale,
                curve: EasingCurve::new(0.0, 1.0, ease_fn),
                callback_entity,
            };
            animations.zoom = Some(anim);
        }
        CameraControlType::Instant => {
            projection.scale = event.scale;
        }
    }
}

pub fn animate_camera_zoom(
    mut commands: Commands,
    mut animations: ResMut<CameraAnimations>,
    time: Res<Time>,
    mut camera: Single<&mut Projection, With<MainCamera>>,
) {
    let Some(ref mut anim) = animations.zoom else {
        return;
    };
    let Projection::Orthographic(ref mut projection) = **camera else {
        panic!("Camera must have orthographic projection");
    };

    anim.progress.tick(time.delta());

    let percent = anim.progress.elapsed_secs() / anim.progress.duration().as_secs_f32();
    projection.scale = anim
        .start
        .lerp(anim.end, anim.curve.sample_clamped(percent));

    if anim.progress.just_finished() {
        if let Some(callback_entity) = anim.callback_entity {
            commands.trigger(Callback {
                entity: callback_entity,
            });
        }
        animations.zoom = None;
    }
}
