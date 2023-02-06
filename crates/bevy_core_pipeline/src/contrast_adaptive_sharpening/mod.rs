use crate::{core_2d, core_3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state};
use bevy_app::prelude::*;
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_ecs::{prelude::*, query::QueryItem};
use bevy_reflect::{Reflect, TypeUuid};
use bevy_render::{
    extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
    prelude::Camera,
    render_graph::RenderGraph,
    render_resource::*,
    renderer::RenderDevice,
    texture::BevyDefault,
    view::{ExtractedView, ViewTarget},
    RenderApp, RenderSet,
};

mod node;

pub use node::CASNode;

/// Applies a contrast adaptive sharpening (CAS) filter to the camera.
///
/// CAS is usually used in combination with shader based anti-aliasing methods
/// such as FXAA or TAA to regain some of the lost detail from the blurring that they introduce.
///
/// CAS is designed to adjust the amount of sharpening applied to different areas of an image
/// based on the local contrast. This can help avoid over-sharpening areas with high contrast
/// and under-sharpening areas with low contrast.
///
/// To use this, add the [`ContrastAdaptiveSharpeningSettings`] component to a 2D or 3D camera.
#[derive(Component, Reflect, Clone)]
pub struct ContrastAdaptiveSharpeningSettings {
    /// Enable or disable sharpening.
    pub enabled: bool,
    /// Adjusts how the shader adapts to high contrast.
    /// Higher values = more high contrast sharpening.
    ///
    /// Range of 0.0 to 1.0, with 0.0 not being fully off.
    pub contrast_adaption: f32,
    /// Adjusts sharpening intensity by averaging original pixels to the sharpened result.
    ///
    /// Range of 0.0 to 1.0, with 0.0 being fully off.
    pub sharpening_intensity: f32,
}

/// The uniform struct extracted from [`ContrastAdaptiveSharpeningSettings`] attached to a [`Camera`].
/// Will be available for use in the CAS shader.
#[doc(hidden)]
#[derive(Component, ShaderType, Clone)]
pub struct CASUniform {
    contrast_adaption: f32,
    sharpening_intensity: f32,
}

impl Default for ContrastAdaptiveSharpeningSettings {
    fn default() -> Self {
        ContrastAdaptiveSharpeningSettings {
            enabled: true,
            contrast_adaption: 0.1,
            sharpening_intensity: 1.0,
        }
    }
}

impl ExtractComponent for ContrastAdaptiveSharpeningSettings {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = CASUniform;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        if !item.enabled {
            return None;
        }
        Some(CASUniform {
            contrast_adaption: item.contrast_adaption.clamp(0.0, 1.0),
            sharpening_intensity: item.sharpening_intensity.clamp(0.0, 1.0),
        })
    }
}

const CONTRAST_ADAPTIVE_SHARPENING_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6925381244141981602);

/// Adds Support for Contrast Adaptive Sharpening (CAS).
pub struct CASPlugin;

impl Plugin for CASPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            CONTRAST_ADAPTIVE_SHARPENING_SHADER_HANDLE,
            "contrast_adaptive_sharpening.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<ContrastAdaptiveSharpeningSettings>();
        app.add_plugin(ExtractComponentPlugin::<ContrastAdaptiveSharpeningSettings>::default());
        app.add_plugin(UniformComponentPlugin::<CASUniform>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app
            .init_resource::<CASPipeline>()
            .init_resource::<SpecializedRenderPipelines<CASPipeline>>()
            .add_system(prepare_cas_pipelines.in_set(RenderSet::Prepare));
        {
            let cas_node = CASNode::new(&mut render_app.world);
            let mut binding = render_app.world.resource_mut::<RenderGraph>();
            let graph = binding.get_sub_graph_mut(core_3d::graph::NAME).unwrap();

            graph.add_node(core_3d::graph::node::CONTRAST_ADAPTIVE_SHARPENING, cas_node);

            graph.add_slot_edge(
                graph.input_node().id,
                core_3d::graph::input::VIEW_ENTITY,
                core_3d::graph::node::CONTRAST_ADAPTIVE_SHARPENING,
                CASNode::IN_VIEW,
            );

            graph.add_node_edge(
                core_3d::graph::node::FXAA,
                core_3d::graph::node::CONTRAST_ADAPTIVE_SHARPENING,
            );
            graph.add_node_edge(
                core_3d::graph::node::CONTRAST_ADAPTIVE_SHARPENING,
                core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
            );
        }
        {
            let cas_node = CASNode::new(&mut render_app.world);
            let mut binding = render_app.world.resource_mut::<RenderGraph>();
            let graph = binding.get_sub_graph_mut(core_2d::graph::NAME).unwrap();

            graph.add_node(core_2d::graph::node::CONTRAST_ADAPTIVE_SHARPENING, cas_node);

            graph.add_slot_edge(
                graph.input_node().id,
                core_2d::graph::input::VIEW_ENTITY,
                core_2d::graph::node::CONTRAST_ADAPTIVE_SHARPENING,
                CASNode::IN_VIEW,
            );

            graph.add_node_edge(
                core_2d::graph::node::FXAA,
                core_2d::graph::node::CONTRAST_ADAPTIVE_SHARPENING,
            );
            graph.add_node_edge(
                core_2d::graph::node::CONTRAST_ADAPTIVE_SHARPENING,
                core_2d::graph::node::END_MAIN_PASS_POST_PROCESSING,
            );
        }
    }
}

#[derive(Resource)]
pub struct CASPipeline {
    texture_bind_group: BindGroupLayout,
    sampler: Sampler,
}

impl FromWorld for CASPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let render_device = render_world.resource::<RenderDevice>();
        let texture_bind_group =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("sharpening_texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    // CAS Settings
                    BindGroupLayoutEntry {
                        binding: 2,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(CASUniform::min_size()),
                        },
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                    },
                ],
            });

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        CASPipeline {
            texture_bind_group,
            sampler,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct CASPipelineKey {
    texture_format: TextureFormat,
}

impl SpecializedRenderPipeline for CASPipeline {
    type Key = CASPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("contrast_adaptive_sharpening".into()),
            layout: Some(vec![self.texture_bind_group.clone()]),
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: CONTRAST_ADAPTIVE_SHARPENING_SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        }
    }
}

pub fn prepare_cas_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CASPipeline>>,
    sharpening_pipeline: Res<CASPipeline>,
    views: Query<(Entity, &ExtractedView, &CASUniform)>,
) {
    for (entity, view, sharpening) in &views {
        if sharpening.sharpening_intensity == 0.0 {
            continue;
        }
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &sharpening_pipeline,
            CASPipelineKey {
                texture_format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                },
            },
        );

        commands.entity(entity).insert(ViewCASPipeline(pipeline_id));
    }
}

#[derive(Component)]
pub struct ViewCASPipeline(CachedRenderPipelineId);
