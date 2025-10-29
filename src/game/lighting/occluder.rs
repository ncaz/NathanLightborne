use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{add_visibility_class, VisibilityClass, VisibilitySystems},
    },
    core_pipeline::FullscreenShader,
    ecs::{
        query::{QueryItem, ROQueryItem},
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    math::{vec3, Affine3, Affine3A},
    mesh::VertexBufferLayout,
    platform::collections::HashMap,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass,
            ViewSortedRenderPhases,
        },
        render_resource::{binding_types::uniform_buffer, *},
        renderer::{RenderDevice, RenderQueue},
        texture::TextureCache,
        view::{ExtractedView, ViewDepthTexture, ViewTarget},
        Render, RenderApp, RenderStartup, RenderSystems,
    },
    shader::ShaderDefVal,
    sprite_render::{init_mesh_2d_pipeline, Mesh2dPipeline},
};
use bytemuck::{Pod, Zeroable};

use crate::game::lighting::render::{post_process_layout, DeferredLighting2d};

use super::{
    line_light::{line_light_bind_group_layout, LineLight2dBounds},
    AmbientLight2d,
};

pub struct Occluder2dPipelinePlugin;

impl Plugin for Occluder2dPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UniformComponentPlugin::<ExtractOccluder2d>::default())
            .add_plugins(ExtractComponentPlugin::<Occluder2d>::default())
            .add_plugins(ExtractComponentPlugin::<Occluder2dGroups>::default())
            .add_systems(
                PostUpdate,
                (calculate_occluder_2d_bounds.in_set(VisibilitySystems::CalculateBounds),),
            );

        let shader: Handle<Shader> = app.world().load_asset("shaders/lighting/occluder.wgsl");
        let reset_shader: Handle<Shader> = app
            .world()
            .load_asset("shaders/lighting/occluder_reset.wgsl");

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            Render,
            prepare_occluder_count_textures.in_set(RenderSystems::PrepareResources),
        );
        render_app.add_systems(
            Render,
            prepare_occluder_2d_bind_group.in_set(RenderSystems::PrepareBindGroups),
        );
        render_app.insert_resource(Occluder2dAssets {
            shader,
            reset_shader,
        });
        render_app.add_systems(
            RenderStartup,
            init_occluder_2d_pipeline.after(init_mesh_2d_pipeline),
        );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<Occluder2dBuffers>();
    }
}

/// Add to line lights and occluders to mark which occluders should occlude which line lights.
/// An occluder will only occlude a line light if (line light's occluder mask) & (occluder
/// occluder mask) is not zero.
#[derive(Component, ExtractComponent, Clone, Copy, PartialEq, Eq)]
pub struct Occluder2dGroups(pub u32);

impl Occluder2dGroups {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(!0);

    pub fn _group(layer: u32) -> Self {
        Self(1 << layer)
    }

    pub fn _from_groups(layers: &[u32]) -> Self {
        let mut mask = 0;
        for i in layers {
            mask |= 1 << i;
        }
        Self(mask)
    }
}

impl Default for Occluder2dGroups {
    fn default() -> Self {
        Self::ALL
    }
}

#[derive(Component)]
#[require(Transform, Visibility, Occluder2dGroups, VisibilityClass)]
#[component(on_add = add_visibility_class::<Occluder2d>)]
pub struct Occluder2d {
    pub half_size: Vec2,
}

impl Occluder2d {
    pub fn new(half_x: f32, half_y: f32) -> Self {
        Self {
            half_size: Vec2::new(half_x, half_y),
        }
    }
}

pub fn calculate_occluder_2d_bounds(
    mut commands: Commands,
    q_light_changed: Query<(Entity, &Occluder2d), Changed<Occluder2d>>,
) {
    for (entity, occluder) in q_light_changed.iter() {
        let aabb = Aabb {
            center: Vec3::ZERO.into(),
            half_extents: occluder.half_size.extend(0.0).into(),
        };
        commands.entity(entity).try_insert(aabb);
    }
}

impl ExtractComponent for Occluder2d {
    type Out = (ExtractOccluder2d, Occluder2dBounds);
    type QueryData = (
        &'static GlobalTransform,
        &'static Occluder2d,
        Has<Occluder2dDisabled>,
    );
    type QueryFilter = ();

    fn extract_component(
        (transform, occluder, disabled): QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        if disabled {
            return None;
        }
        // FIXME: should not do calculations in extract
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        let transform_no_scale =
            Affine3A::from_scale_rotation_translation(scale.signum(), rotation, translation);
        let affine = Affine3::from(&transform_no_scale);
        let (a, b) = affine.inverse_transpose_3x3();

        Some((
            ExtractOccluder2d {
                world_from_local: affine.to_transpose(),
                local_from_world_transpose_a: a,
                local_from_world_transpose_b: b,
                half_size: occluder.half_size,
            },
            Occluder2dBounds {
                transform: transform.compute_transform(),
                half_size: occluder.half_size,
            },
        ))
    }
}

/// Render world version of [`Occluder2d`].
#[derive(Component, ShaderType, Clone, Copy, Debug)]
pub struct ExtractOccluder2d {
    world_from_local: [Vec4; 3],
    local_from_world_transpose_a: [Vec4; 2],
    local_from_world_transpose_b: f32,
    half_size: Vec2,
}

#[derive(Component, Clone, Copy)]
pub struct Occluder2dBounds {
    pub transform: Transform,
    pub half_size: Vec2,
}

#[derive(Component)]
pub struct Occluder2dDisabled;

impl Occluder2dBounds {
    pub fn visible_from_line_light(&self, light: &LineLight2dBounds) -> bool {
        let occluder_pos = self.transform.translation.xy();
        let min_rect = occluder_pos - self.half_size;
        let max_rect = occluder_pos + self.half_size;

        let light_pos = light.transform.translation.xy();
        let closest_point = light_pos.clamp(min_rect, max_rect);

        light_pos.distance_squared(closest_point)
            <= (light.radius + light.half_length) * (light.radius + light.half_length)
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Occluder2dVertex {
    position: Vec3,
    normal: Vec3,
}

impl Occluder2dVertex {
    const fn new(position: Vec3, normal: Vec3) -> Self {
        Occluder2dVertex { position, normal }
    }
}

#[derive(Resource)]
pub struct Occluder2dBuffers {
    pub vertices: RawBufferVec<Occluder2dVertex>,
    pub indices: RawBufferVec<u32>,
}

const OCCLUDER_2D_NUM_INDICES: u32 = 18;

static VERTICES: [Occluder2dVertex; 8] = [
    Occluder2dVertex::new(vec3(-1.0, -1.0, 0.0), vec3(-1.0, 0.0, 0.0)),
    Occluder2dVertex::new(vec3(-1.0, -1.0, 0.0), vec3(0.0, -1.0, 0.0)),
    Occluder2dVertex::new(vec3(1.0, -1.0, 0.0), vec3(0.0, -1.0, 0.0)),
    Occluder2dVertex::new(vec3(1.0, -1.0, 0.0), vec3(1.0, 0.0, 0.0)),
    Occluder2dVertex::new(vec3(1.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0)),
    Occluder2dVertex::new(vec3(1.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0)),
    Occluder2dVertex::new(vec3(-1.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0)),
    Occluder2dVertex::new(vec3(-1.0, 1.0, 0.0), vec3(-1.0, 0.0, 0.0)),
];

static INDICES: [u32; 18] = [0, 1, 2, 2, 3, 4, 4, 5, 6, 6, 7, 0, 0, 2, 4, 4, 6, 0];

impl FromWorld for Occluder2dBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut vbo = RawBufferVec::new(BufferUsages::VERTEX);
        let mut ibo = RawBufferVec::new(BufferUsages::INDEX);

        for vtx in &VERTICES {
            vbo.push(*vtx);
        }
        for index in &INDICES {
            ibo.push(*index);
        }

        vbo.write_buffer(render_device, render_queue);
        ibo.write_buffer(render_device, render_queue);

        Occluder2dBuffers {
            vertices: vbo,
            indices: ibo,
        }
    }
}

#[derive(Component)]
pub struct OccluderCountTexture(pub ViewDepthTexture);

/// Prepare my own texture because theirs has funny sample count??
#[allow(clippy::type_complexity)]
pub fn prepare_occluder_count_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    deferred_lighting_phases: Res<ViewSortedRenderPhases<DeferredLighting2d>>,
    views: Query<
        (Entity, &ExtractedCamera, &ExtractedView),
        (With<Camera2d>, With<AmbientLight2d>),
    >,
) {
    let mut textures = <HashMap<_, _>>::default();
    for (view, camera, extract_view) in &views {
        if !deferred_lighting_phases.contains_key(&extract_view.retained_view_entity) {
            continue;
        }
        let Some(physical_target_size) = camera.physical_target_size else {
            continue;
        };

        let cached_texture = textures
            .entry(camera.target.clone())
            .or_insert_with(|| {
                // The size of the depth texture
                let size = Extent3d {
                    depth_or_array_layers: 1,
                    width: physical_target_size.x,
                    height: physical_target_size.y,
                };

                let descriptor = TextureDescriptor {
                    label: Some("occluder_count_texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Stencil8,
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                };

                texture_cache.get(&render_device, descriptor)
            })
            .clone();

        commands
            .entity(view)
            .insert(OccluderCountTexture(ViewDepthTexture::new(
                cached_texture,
                Some(0.0),
            )));
    }
}

#[derive(Resource)]
pub struct Occluder2dBindGroup {
    value: BindGroup,
}

pub fn prepare_occluder_2d_bind_group(
    mut commands: Commands,
    uniforms: Res<ComponentUniforms<ExtractOccluder2d>>,
    pipeline: Res<Occluder2dPipeline>,
    render_device: Res<RenderDevice>,
) {
    if let Some(binding) = uniforms.uniforms().binding() {
        commands.insert_resource(Occluder2dBindGroup {
            value: render_device.create_bind_group(
                "occluder_2d_bind_group",
                &pipeline.layout,
                &BindGroupEntries::single(binding),
            ),
        })
    }
}

pub struct SetOccluder2dBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetOccluder2dBindGroup<I> {
    type Param = SRes<Occluder2dBindGroup>;
    type ViewQuery = ();
    type ItemQuery = Read<DynamicUniformIndex<ExtractOccluder2d>>;

    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, '_, Self::ViewQuery>,
        entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(index) = entity else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &param.into_inner().value, &[index.index()]);
        RenderCommandResult::Success
    }
}

pub struct DrawOccluder2d;
impl<P: PhaseItem> RenderCommand<P> for DrawOccluder2d {
    type Param = SRes<Occluder2dBuffers>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let buffers = param.into_inner();

        pass.set_vertex_buffer(0, buffers.vertices.buffer().unwrap().slice(..));
        pass.set_index_buffer(
            buffers.indices.buffer().unwrap().slice(..),
            0,
            IndexFormat::Uint32,
        );
        pass.draw_indexed(0..OCCLUDER_2D_NUM_INDICES, 0, 0..1);

        RenderCommandResult::Success
    }
}

#[derive(Resource)]
pub struct Occluder2dAssets {
    shader: Handle<Shader>,
    reset_shader: Handle<Shader>,
}

#[derive(Resource)]
pub struct Occluder2dPipeline {
    pub layout: BindGroupLayout,
    pub shadow_pipeline_id: CachedRenderPipelineId,
    pub cutout_pipeline_id: CachedRenderPipelineId,
    pub reset_pipeline_id: CachedRenderPipelineId,
}

pub fn build_occluder_2d_pipeline_descriptor(
    render_device: &Res<RenderDevice>,
    occluder_2d_assets: &Res<Occluder2dAssets>,
    mesh2d_pipeline: &Res<Mesh2dPipeline>,
    cutout: bool,
    occluder_layout: BindGroupLayout,
) -> RenderPipelineDescriptor {
    let post_process_layout = post_process_layout(render_device);
    let line_light_layout = line_light_bind_group_layout(render_device);
    let shader = occluder_2d_assets.shader.clone();

    let pos_buffer_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<Occluder2dVertex>() as u64,
        step_mode: VertexStepMode::Vertex,
        attributes: vec![
            // Position
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: std::mem::offset_of!(Occluder2dVertex, position) as u64,
                shader_location: 0,
            },
            // Normals
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: std::mem::offset_of!(Occluder2dVertex, normal) as u64,
                shader_location: 1,
            },
        ],
    };

    let mut shader_defs: Vec<ShaderDefVal> = vec![];
    if cutout {
        shader_defs.push("OCCLUDER_CUTOUT".into());
    }

    let label = if cutout {
        Some("occluder_cutout_pipeline".into())
    } else {
        Some("occluder_pipeline".into())
    };

    RenderPipelineDescriptor {
        label,
        layout: vec![
            post_process_layout,
            mesh2d_pipeline.view_layout.clone(),
            line_light_layout,
            occluder_layout,
        ],
        vertex: VertexState {
            shader: shader.clone(),
            shader_defs: shader_defs.clone(),
            entry_point: Some("vertex".into()),
            buffers: vec![pos_buffer_layout],
        },
        fragment: Some(FragmentState {
            shader,
            shader_defs,
            entry_point: Some("fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: ViewTarget::TEXTURE_FORMAT_HDR,
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::Zero,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                }),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Stencil8,
            depth_write_enabled: false,
            depth_compare: CompareFunction::Always,
            stencil: StencilState {
                front: StencilFaceState {
                    compare: CompareFunction::Always,
                    fail_op: StencilOperation::Keep,
                    depth_fail_op: StencilOperation::Keep,
                    pass_op: if cutout {
                        StencilOperation::Zero
                    } else {
                        StencilOperation::IncrementClamp
                    },
                },
                back: StencilFaceState::default(),
                read_mask: 0xFF,
                write_mask: 0xFF,
            },
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        push_constant_ranges: vec![],
        zero_initialize_workgroup_memory: false,
    }
}

pub fn init_occluder_2d_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    occluder_2d_assets: Res<Occluder2dAssets>,
    mesh2d_pipeline: Res<Mesh2dPipeline>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: ResMut<PipelineCache>,
) {
    let layout = render_device.create_bind_group_layout(
        "occluder_bind_group_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::VERTEX_FRAGMENT,
            uniform_buffer::<ExtractOccluder2d>(true),
        ),
    );

    let reset_shader = occluder_2d_assets.reset_shader.clone();

    let shadow_pipeline_descriptor = build_occluder_2d_pipeline_descriptor(
        &render_device,
        &occluder_2d_assets,
        &mesh2d_pipeline,
        false,
        layout.clone(),
    );
    let cutout_pipeline_descriptor = build_occluder_2d_pipeline_descriptor(
        &render_device,
        &occluder_2d_assets,
        &mesh2d_pipeline,
        true,
        layout.clone(),
    );

    let vertex_state = fullscreen_shader.to_vertex_state();

    let shadow_pipeline_id = pipeline_cache.queue_render_pipeline(shadow_pipeline_descriptor);
    let cutout_pipeline_id = pipeline_cache.queue_render_pipeline(cutout_pipeline_descriptor);

    let reset_pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("occluder_reset_pipeline".into()),
        layout: vec![],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader: reset_shader,
            shader_defs: vec![],
            entry_point: Some("fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: ViewTarget::TEXTURE_FORMAT_HDR,
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::Zero,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::Zero,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                }),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Stencil8,
            depth_write_enabled: false,
            depth_compare: CompareFunction::Always,
            stencil: StencilState {
                front: StencilFaceState {
                    compare: CompareFunction::Always,
                    fail_op: StencilOperation::Zero,
                    depth_fail_op: StencilOperation::Zero,
                    pass_op: StencilOperation::Zero,
                },
                back: StencilFaceState::default(),
                read_mask: 0xFF,
                write_mask: 0xFF,
            },
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        push_constant_ranges: vec![],
        zero_initialize_workgroup_memory: false,
    });

    commands.insert_resource(Occluder2dPipeline {
        layout,
        shadow_pipeline_id,
        cutout_pipeline_id,
        reset_pipeline_id,
    });
}

// WebGL2 requires thes structs be 16-byte aligned
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn occluder_2d_alignment() {
        assert_eq!(mem::size_of::<ExtractOccluder2d>() % 16, 0);
    }
}
