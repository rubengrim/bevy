// Ground Truth-based Ambient Occlusion (GTAO)
// Paper: https://www.activision.com/cdn/research/Practical_Real_Time_Strategies_for_Accurate_Indirect_Occlusion_NEW%20VERSION_COLOR.pdf
// Presentation: https://blog.selfshadow.com/publications/s2016-shading-course/activision/s2016_pbs_activision_occlusion.pdf

// Source code heavily based on XeGTAO v1.30 from Intel
// https://github.com/GameTechDev/XeGTAO/blob/0d177ce06bfa642f64d8af4de1197ad1bcb862d4/Source/Rendering/Shaders/XeGTAO.hlsli

use crate::MAX_DIRECTIONAL_LIGHTS;
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_core_pipeline::{
    prelude::Camera3d,
    prepass::{PrepassSettings, ViewPrepassTextures},
};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::{QueryState, With},
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_reflect::{Reflect, TypeUuid};
use bevy_render::{
    camera::ExtractedCamera,
    globals::{GlobalsBuffer, GlobalsUniform},
    prelude::Camera,
    render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType},
    render_resource::{
        AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
        BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
        BufferBindingType, CachedComputePipelineId, ComputePassDescriptor,
        ComputePipelineDescriptor, Extent3d, FilterMode, PipelineCache, Sampler,
        SamplerBindingType, SamplerDescriptor, Shader, ShaderDefVal, ShaderStages, ShaderType,
        StorageTextureAccess, TextureDescriptor, TextureDimension, TextureFormat,
        TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue},
    texture::{CachedTexture, TextureCache},
    view::{ViewUniform, ViewUniformOffset, ViewUniforms},
    Extract, RenderApp, RenderStage,
};
#[cfg(feature = "trace")]
use bevy_utils::tracing::info_span;
use bevy_utils::{prelude::default, HashMap};
use std::{mem, num::NonZeroU32};

pub mod draw_3d_graph {
    pub mod node {
        /// Label for the ambient occlusion render node.
        pub const AMBIENT_OCCLUSION: &str = "ambient_occlusion";
    }
}

const PREFILTER_DEPTH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 102258915420479);
const GTAO_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 253938746510568);
const DENOISE_AO_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 466162052558226);
const GTAO_MULTIBOUNCE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 366465052568786);

// TODO: Support MSAA

pub struct AmbientOcclusionPlugin;

impl Plugin for AmbientOcclusionPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PREFILTER_DEPTH_SHADER_HANDLE,
            "prefilter_depth.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(app, GTAO_SHADER_HANDLE, "gtao.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            DENOISE_AO_SHADER_HANDLE,
            "denoise_ao.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            GTAO_MULTIBOUNCE_SHADER_HANDLE,
            "gtao_multibounce.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<AmbientOcclusionSettings>();

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else { return };

        render_app
            .init_resource::<AmbientOcclusionPipelines>()
            .add_system_to_stage(RenderStage::Extract, extract_ambient_occlusion_settings)
            .add_system_to_stage(RenderStage::Prepare, prepare_ambient_occlusion_textures)
            .add_system_to_stage(RenderStage::Queue, queue_ambient_occlusion_bind_groups);

        let ao_node = AmbientOcclusionNode::new(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let draw_3d_graph = graph
            .get_sub_graph_mut(bevy_core_pipeline::core_3d::graph::NAME)
            .unwrap();
        draw_3d_graph.add_node(draw_3d_graph::node::AMBIENT_OCCLUSION, ao_node);
        draw_3d_graph.add_slot_edge(
            draw_3d_graph.input_node().id,
            bevy_core_pipeline::core_3d::graph::input::VIEW_ENTITY,
            draw_3d_graph::node::AMBIENT_OCCLUSION,
            AmbientOcclusionNode::IN_VIEW,
        );
        // PREPASS -> AMBIENT_OCCLUSION -> MAIN_PASS
        draw_3d_graph.add_node_edge(
            bevy_core_pipeline::core_3d::graph::node::PREPASS,
            draw_3d_graph::node::AMBIENT_OCCLUSION,
        );
        draw_3d_graph.add_node_edge(
            draw_3d_graph::node::AMBIENT_OCCLUSION,
            bevy_core_pipeline::core_3d::graph::node::MAIN_PASS,
        );
    }
}

#[derive(Component, Reflect, Clone)]
pub enum AmbientOcclusionSettings {
    Low,
    Medium,
    High,
    Ultra,
}

impl Default for AmbientOcclusionSettings {
    fn default() -> Self {
        Self::Medium
    }
}

impl AmbientOcclusionSettings {
    fn sample_counts(&self) -> (u32, u32) {
        match self {
            AmbientOcclusionSettings::Low => (1, 2), // 4 spp (1 * (2 * 2)), plus optional temporal samples
            AmbientOcclusionSettings::Medium => (2, 2), // 8 spp (2 * (2 * 2)), plus optional temporal samples
            AmbientOcclusionSettings::High => (3, 3), // 18 spp (3 * (3 * 2)), plus optional temporal samples
            AmbientOcclusionSettings::Ultra => (9, 3), // 54 spp (9 * (3 * 2)), plus optional temporal samples
        }
    }
}

struct AmbientOcclusionNode {
    view_query: QueryState<(
        &'static ExtractedCamera,
        &'static AmbientOcclusionBindGroups,
        &'static ViewUniformOffset,
    )>,
}

impl AmbientOcclusionNode {
    const IN_VIEW: &'static str = "view";

    fn new(world: &mut World) -> Self {
        Self {
            view_query: QueryState::new(world),
        }
    }
}

impl Node for AmbientOcclusionNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.view_query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        #[cfg(feature = "trace")]
        let _ao_span = info_span!("ambient_occlusion").entered();

        let pipelines = world.resource::<AmbientOcclusionPipelines>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (
            Ok((camera, bind_groups, view_uniform_offset)),
            Some(prefilter_depth_pipeline),
            Some(gtao_pipeline),
            Some(denoise_pipeline),
        ) = (
            self.view_query.get_manual(world, view_entity),
            pipeline_cache.get_compute_pipeline(pipelines.prefilter_depth_pipeline),
            pipeline_cache.get_compute_pipeline(pipelines.gtao_pipeline),
            pipeline_cache.get_compute_pipeline(pipelines.denoise_pipeline),
        ) else {
            return Ok(());
        };
        let Some(camera_size) = camera.physical_viewport_size else { return Ok(()) };

        {
            let mut prefilter_depth_pass =
                render_context
                    .command_encoder
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("ambient_occlusion_prefilter_depth_pass"),
                    });
            prefilter_depth_pass.set_pipeline(prefilter_depth_pipeline);
            prefilter_depth_pass.set_bind_group(0, &bind_groups.prefilter_depth_bind_group, &[]);
            prefilter_depth_pass.set_bind_group(
                1,
                &bind_groups.common_bind_group,
                &[view_uniform_offset.offset],
            );
            prefilter_depth_pass.dispatch_workgroups(
                (camera_size.x + 15) / 16,
                (camera_size.y + 15) / 16,
                1,
            );
        }

        {
            let mut gtao_pass =
                render_context
                    .command_encoder
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("ambient_occlusion_gtao_pass"),
                    });
            gtao_pass.set_pipeline(gtao_pipeline);
            gtao_pass.set_bind_group(0, &bind_groups.gtao_bind_group, &[]);
            gtao_pass.set_bind_group(
                1,
                &bind_groups.common_bind_group,
                &[view_uniform_offset.offset],
            );
            gtao_pass.dispatch_workgroups((camera_size.x + 7) / 8, (camera_size.y + 7) / 8, 1);
        }

        {
            let mut denoise_pass =
                render_context
                    .command_encoder
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("ambient_occlusion_denoise_pass"),
                    });
            denoise_pass.set_pipeline(&denoise_pipeline);
            denoise_pass.set_bind_group(0, &bind_groups.denoise_bind_group, &[]);
            denoise_pass.set_bind_group(
                1,
                &bind_groups.common_bind_group,
                &[view_uniform_offset.offset],
            );
            denoise_pass.dispatch_workgroups((camera_size.x + 7) / 8, (camera_size.y + 7) / 8, 1);
        }

        Ok(())
    }
}

#[derive(Resource)]
struct AmbientOcclusionPipelines {
    prefilter_depth_pipeline: CachedComputePipelineId,
    gtao_pipeline: CachedComputePipelineId,
    denoise_pipeline: CachedComputePipelineId,

    common_bind_group_layout: BindGroupLayout,
    prefilter_depth_bind_group_layout: BindGroupLayout,
    gtao_bind_group_layout: BindGroupLayout,
    denoise_bind_group_layout: BindGroupLayout,

    hilbert_index_texture: TextureView,
    point_clamp_sampler: Sampler,
}

impl FromWorld for AmbientOcclusionPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let hilbert_index_texture = render_device
            .create_texture_with_data(
                render_queue,
                &(TextureDescriptor {
                    label: Some("ambient_occlusion_hilbert_index_texture"),
                    size: Extent3d {
                        width: 64,
                        height: 64,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::R16Uint,
                    usage: TextureUsages::TEXTURE_BINDING,
                }),
                bytemuck::cast_slice(&generate_hilbert_index_texture()),
            )
            .create_view(&TextureViewDescriptor::default());

        let point_clamp_sampler = render_device.create_sampler(&SamplerDescriptor {
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..Default::default()
        });

        let common_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("ambient_occlusion_common_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(ViewUniform::min_size()),
                        },
                        count: None,
                    },
                ],
            });

        let mip_texture_entry = BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                format: TextureFormat::R32Float,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        };
        let prefilter_depth_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("ambient_occlusion_prefilter_depth_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    mip_texture_entry,
                    BindGroupLayoutEntry {
                        binding: 2,
                        ..mip_texture_entry
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        ..mip_texture_entry
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        ..mip_texture_entry
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        ..mip_texture_entry
                    },
                ],
            });

        let gtao_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("ambient_occlusion_gtao_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Uint,
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::R32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::R32Uint,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(GlobalsUniform::min_size()),
                        },
                        count: None,
                    },
                ],
            });

        let denoise_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("ambient_occlusion_denoise_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Uint,
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::R32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let prefilter_depth_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ambient_occlusion_prefilter_depth_pipeline".into()),
                layout: Some(vec![
                    prefilter_depth_bind_group_layout.clone(),
                    common_bind_group_layout.clone(),
                ]),
                shader: PREFILTER_DEPTH_SHADER_HANDLE.typed(),
                shader_defs: vec![ShaderDefVal::Int(
                    // TODO: Remove this hack
                    "MAX_DIRECTIONAL_LIGHTS".to_string(),
                    MAX_DIRECTIONAL_LIGHTS as i32,
                )],
                entry_point: "prefilter_depth".into(),
            });

        let gtao_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ambient_occlusion_gtao_pipeline".into()),
            layout: Some(vec![
                gtao_bind_group_layout.clone(),
                common_bind_group_layout.clone(),
            ]),
            shader: GTAO_SHADER_HANDLE.typed(),
            shader_defs: vec![
                // TODO: Specalize based on AmbientOcclusionSettings (and TemporalAntiAliasSettings)
                "TEMPORAL_NOISE".into(),
                ShaderDefVal::Int("SLICE_COUNT".to_string(), 3),
                ShaderDefVal::Int("SAMPLES_PER_SLICE_SIDE".to_string(), 3),
                // TODO: Remove this hack
                ShaderDefVal::Int(
                    "MAX_DIRECTIONAL_LIGHTS".to_string(),
                    MAX_DIRECTIONAL_LIGHTS as i32,
                ),
            ],
            entry_point: "gtao".into(),
        });

        let denoise_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ambient_occlusion_denoise_pipeline".into()),
            layout: Some(vec![
                denoise_bind_group_layout.clone(),
                common_bind_group_layout.clone(),
            ]),
            shader: DENOISE_AO_SHADER_HANDLE.typed(),
            shader_defs: vec![
                // TODO: Remove this hack
                ShaderDefVal::Int(
                    "MAX_DIRECTIONAL_LIGHTS".to_string(),
                    MAX_DIRECTIONAL_LIGHTS as i32,
                ),
            ],
            entry_point: "denoise".into(),
        });

        Self {
            prefilter_depth_pipeline,
            gtao_pipeline,
            denoise_pipeline,

            common_bind_group_layout,
            prefilter_depth_bind_group_layout,
            gtao_bind_group_layout,
            denoise_bind_group_layout,

            hilbert_index_texture,
            point_clamp_sampler,
        }
    }
}

fn extract_ambient_occlusion_settings(
    mut commands: Commands,
    cameras: Extract<
        Query<(Entity, &Camera, &AmbientOcclusionSettings, &PrepassSettings), With<Camera3d>>,
    >,
) {
    for (entity, camera, ao_settings, prepass_settings) in &cameras {
        if camera.is_active && prepass_settings.depth_enabled && prepass_settings.normal_enabled {
            commands.get_or_spawn(entity).insert(ao_settings.clone());
        }
    }
}

#[derive(Component)]
pub struct AmbientOcclusionTextures {
    prefiltered_depth_texture: CachedTexture,
    ambient_occlusion_noisy_texture: CachedTexture, // Pre-denoised texture
    pub ambient_occlusion_texture: CachedTexture,   // Denoised texture
    depth_differences_texture: CachedTexture,
}

fn prepare_ambient_occlusion_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera), With<AmbientOcclusionSettings>>,
) {
    let mut prefiltered_depth_textures = HashMap::default();
    let mut ambient_occlusion_noisy_textures = HashMap::default();
    let mut ambient_occlusion_textures = HashMap::default();
    let mut depth_differences_textures = HashMap::default();
    for (entity, camera) in &views {
        if let Some(physical_viewport_size) = camera.physical_viewport_size {
            let size = Extent3d {
                width: physical_viewport_size.x,
                height: physical_viewport_size.y,
                depth_or_array_layers: 1,
            };

            let texture_descriptor = TextureDescriptor {
                label: Some("ambient_occlusion_prefiltered_depth_texture"),
                size,
                mip_level_count: 5,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Float,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            };
            let prefiltered_depth_texture = prefiltered_depth_textures
                .entry(camera.target.clone())
                .or_insert_with(|| texture_cache.get(&render_device, texture_descriptor))
                .clone();

            let texture_descriptor = TextureDescriptor {
                label: Some("ambient_occlusion_noisy_texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Float,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            };
            let ambient_occlusion_noisy_texture = ambient_occlusion_noisy_textures
                .entry(camera.target.clone())
                .or_insert_with(|| texture_cache.get(&render_device, texture_descriptor.clone()))
                .clone();

            let texture_descriptor = TextureDescriptor {
                label: Some("ambient_occlusion_texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Float,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            };
            let ambient_occlusion_texture = ambient_occlusion_textures
                .entry(camera.target.clone())
                .or_insert_with(|| texture_cache.get(&render_device, texture_descriptor.clone()))
                .clone();

            let texture_descriptor = TextureDescriptor {
                label: Some("ambient_occlusion_depth_differences_texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Uint,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            };
            let depth_differences_texture = depth_differences_textures
                .entry(camera.target.clone())
                .or_insert_with(|| texture_cache.get(&render_device, texture_descriptor))
                .clone();

            commands.entity(entity).insert(AmbientOcclusionTextures {
                prefiltered_depth_texture,
                ambient_occlusion_noisy_texture,
                ambient_occlusion_texture,
                depth_differences_texture,
            });
        }
    }
}

#[derive(Component)]
struct AmbientOcclusionBindGroups {
    common_bind_group: BindGroup,
    prefilter_depth_bind_group: BindGroup,
    gtao_bind_group: BindGroup,
    denoise_bind_group: BindGroup,
}

fn queue_ambient_occlusion_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipelines: Res<AmbientOcclusionPipelines>,
    view_uniforms: Res<ViewUniforms>,
    global_uniforms: Res<GlobalsBuffer>,
    views: Query<(Entity, &AmbientOcclusionTextures, &ViewPrepassTextures)>,
) {
    let (Some(view_uniforms), Some(globals_uniforms)) = (
        view_uniforms.uniforms.binding(),
        global_uniforms.buffer.binding(),
    ) else {
        return;
    };

    for (entity, ao_textures, prepass_textures) in &views {
        let common_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("ambient_occlusion_common_bind_group"),
            layout: &pipelines.common_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&pipelines.point_clamp_sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: view_uniforms.clone(),
                },
            ],
        });

        let prefilter_depth_mip_view_descriptor = TextureViewDescriptor {
            format: Some(TextureFormat::R32Float),
            dimension: Some(TextureViewDimension::D2),
            mip_level_count: NonZeroU32::new(1),
            ..default()
        };
        let prefilter_depth_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("ambient_occlusion_prefilter_depth_bind_group"),
            layout: &pipelines.prefilter_depth_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &prepass_textures.depth.as_ref().unwrap().default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &ao_textures.prefiltered_depth_texture.texture.create_view(
                            &TextureViewDescriptor {
                                label: Some(
                                    "ambient_occlusion_prefiltered_depth_texture_mip_view_0",
                                ),
                                base_mip_level: 0,
                                ..prefilter_depth_mip_view_descriptor
                            },
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &ao_textures.prefiltered_depth_texture.texture.create_view(
                            &TextureViewDescriptor {
                                label: Some(
                                    "ambient_occlusion_prefiltered_depth_texture_mip_view_1",
                                ),
                                base_mip_level: 1,
                                ..prefilter_depth_mip_view_descriptor
                            },
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(
                        &ao_textures.prefiltered_depth_texture.texture.create_view(
                            &TextureViewDescriptor {
                                label: Some(
                                    "ambient_occlusion_prefiltered_depth_texture_mip_view_2",
                                ),
                                base_mip_level: 2,
                                ..prefilter_depth_mip_view_descriptor
                            },
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(
                        &ao_textures.prefiltered_depth_texture.texture.create_view(
                            &TextureViewDescriptor {
                                label: Some(
                                    "ambient_occlusion_prefiltered_depth_texture_mip_view_3",
                                ),
                                base_mip_level: 3,
                                ..prefilter_depth_mip_view_descriptor
                            },
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(
                        &ao_textures.prefiltered_depth_texture.texture.create_view(
                            &TextureViewDescriptor {
                                label: Some(
                                    "ambient_occlusion_prefiltered_depth_texture_mip_view_4",
                                ),
                                base_mip_level: 4,
                                ..prefilter_depth_mip_view_descriptor
                            },
                        ),
                    ),
                },
            ],
        });

        let gtao_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("ambient_occlusion_gtao_bind_group"),
            layout: &pipelines.gtao_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &ao_textures.prefiltered_depth_texture.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &prepass_textures.normal.as_ref().unwrap().default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&pipelines.hilbert_index_texture),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(
                        &ao_textures.ambient_occlusion_noisy_texture.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(
                        &ao_textures.depth_differences_texture.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: globals_uniforms.clone(),
                },
            ],
        });

        let denoise_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("ambient_occlusion_denoise_bind_group"),
            layout: &pipelines.denoise_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &ao_textures.ambient_occlusion_noisy_texture.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &ao_textures.depth_differences_texture.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &ao_textures.ambient_occlusion_texture.default_view,
                    ),
                },
            ],
        });

        commands.entity(entity).insert(AmbientOcclusionBindGroups {
            common_bind_group,
            prefilter_depth_bind_group,
            gtao_bind_group,
            denoise_bind_group,
        });
    }
}

fn generate_hilbert_index_texture() -> [[u16; 64]; 64] {
    let mut t = [[0; 64]; 64];

    for x in 0..64 {
        for y in 0..64 {
            t[x][y] = hilbert_index(x as u16, y as u16);
        }
    }

    t
}

// https://www.shadertoy.com/view/3tB3z3
const HILBERT_WIDTH: u16 = 1 << 6;
fn hilbert_index(mut x: u16, mut y: u16) -> u16 {
    let mut index = 0;

    let mut level: u16 = HILBERT_WIDTH / 2;
    while level > 0 {
        let region_x = (x & level > 0) as u16;
        let region_y = (y & level > 0) as u16;
        index += level * level * ((3 * region_x) ^ region_y);

        if region_y == 0 {
            if region_x == 1 {
                x = HILBERT_WIDTH - 1 - x;
                y = HILBERT_WIDTH - 1 - y;
            }

            mem::swap(&mut x, &mut y);
        }

        level /= 2;
    }

    index
}
