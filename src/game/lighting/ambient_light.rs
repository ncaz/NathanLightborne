use bevy::{
    core_pipeline::FullscreenShader,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::{binding_types::uniform_buffer, *},
        renderer::RenderDevice,
        view::ViewTarget,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
    sprite_render::{init_mesh_2d_pipeline, Mesh2dPipeline},
};

use crate::game::lighting::render::post_process_layout;

pub struct AmbientLight2dPlugin;

impl Plugin for AmbientLight2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<AmbientLight2d>::default())
            .add_plugins(UniformComponentPlugin::<AmbientLight2d>::default());

        let shader: Handle<Shader> = app
            .world()
            .load_asset("shaders/lighting/ambient_light.wgsl");

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_systems(
            Render,
            prepare_ambient_light_2d_bind_group.in_set(RenderSystems::PrepareBindGroups),
        );
        render_app.insert_resource(AmbientLight2dAssets { shader });
        render_app.add_systems(
            RenderStartup,
            init_ambient_light_2d_pipeline.after(init_mesh_2d_pipeline),
        );
    }
}

/// Despite its poor name, cameras must have this component to enable deferred lighting.
#[derive(Component, Debug, ExtractComponent, Clone, Copy, ShaderType)]
pub struct AmbientLight2d {
    pub color: Vec4,
}

#[derive(Resource)]
pub struct AmbientLight2dBindGroup {
    value: BindGroup,
}

pub fn prepare_ambient_light_2d_bind_group(
    mut commands: Commands,
    uniforms: Res<ComponentUniforms<AmbientLight2d>>,
    pipeline: Res<AmbientLight2dPipeline>,
    render_device: Res<RenderDevice>,
) {
    if let Some(binding) = uniforms.uniforms().binding() {
        commands.insert_resource(AmbientLight2dBindGroup {
            value: render_device.create_bind_group(
                "ambient_light_2d_bind_group",
                &pipeline.layout,
                &BindGroupEntries::single(binding),
            ),
        })
    }
}

pub struct SetAmbientLight2dBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetAmbientLight2dBindGroup<I> {
    type Param = SRes<AmbientLight2dBindGroup>;
    type ViewQuery = Read<DynamicUniformIndex<AmbientLight2d>>;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &param.into_inner().value, &[view.index()]);
        RenderCommandResult::Success
    }
}

#[derive(Resource)]
pub struct AmbientLight2dAssets {
    shader: Handle<Shader>,
}

#[derive(Resource)]
pub struct AmbientLight2dPipeline {
    pub layout: BindGroupLayout,
    pub pipeline_id: CachedRenderPipelineId,
}

pub fn init_ambient_light_2d_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    ambient_light_2d_assets: Res<AmbientLight2dAssets>,
    mesh2d_pipeline: Res<Mesh2dPipeline>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: ResMut<PipelineCache>,
) {
    let post_process_layout = post_process_layout(&render_device);
    let layout = render_device.create_bind_group_layout(
        "ambient_light_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::FRAGMENT,
            uniform_buffer::<AmbientLight2d>(true),
        ),
    );
    let shader = ambient_light_2d_assets.shader.clone();

    let vertex_state = fullscreen_shader.to_vertex_state();
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("ambient_light_pipeline".into()),
        layout: vec![
            post_process_layout,
            mesh2d_pipeline.view_layout.clone(),
            layout.clone(),
        ],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            shader_defs: vec![],
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
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::Zero,
                        operation: BlendOperation::Add,
                    },
                }),
                write_mask: ColorWrites::ALL,
            })],
        }),
        // below needs changing?
        primitive: PrimitiveState::default(),
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Stencil8,
            depth_write_enabled: false,
            depth_compare: CompareFunction::Always,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        push_constant_ranges: vec![],
        zero_initialize_workgroup_memory: false,
    });

    commands.insert_resource(AmbientLight2dPipeline {
        layout,
        pipeline_id,
    });
}

// WebGL2 requires thes structs be 16-byte aligned
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn ambient_light_2d_alignment() {
        assert_eq!(mem::size_of::<AmbientLight2d>() % 16, 0);
    }
}
