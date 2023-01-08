use crate::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy_app::prelude::*;
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryItem;
use bevy_math::UVec2;
use bevy_reflect::{Reflect, TypeUuid};
use bevy_render::camera::{Camera, ExtractedCamera};
use bevy_render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy_render::renderer::RenderDevice;
use bevy_render::texture::{CachedTexture, TextureCache};
use bevy_render::view::ViewTarget;
use bevy_render::{render_resource::*, RenderApp, RenderStage};
use bevy_utils::default;
use std::array;
use std::num::NonZeroU32;

mod node;

pub use node::TonemappingNode;

const TONEMAPPING_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 17015368199668024512);
const TONEMAPPING_SHARED_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2499430578245347910);
const TONEMAPPING_LOCAL_COMPUTE_LUMINANCES_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1499430578245347910);
const TONEMAPPING_LOCAL_COMPUTE_WEIGHTS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3499430578245347910);
const TONEMAPPING_LOCAL_WEIGH_EXPOSURES_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4499430578245347910);
const TONEMAPPING_LOCAL_BLEND_LAPLACIAN_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5499430578245347910);

pub struct TonemappingPlugin;

impl Plugin for TonemappingPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            TONEMAPPING_SHADER_HANDLE,
            "tonemapping.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            TONEMAPPING_SHARED_SHADER_HANDLE,
            "tonemapping_shared.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            TONEMAPPING_LOCAL_COMPUTE_LUMINANCES_SHADER_HANDLE,
            "local/compute_luminances.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            TONEMAPPING_LOCAL_COMPUTE_WEIGHTS_SHADER_HANDLE,
            "local/compute_weights.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            TONEMAPPING_LOCAL_WEIGH_EXPOSURES_SHADER_HANDLE,
            "local/weigh_exposures.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            TONEMAPPING_LOCAL_BLEND_LAPLACIAN_SHADER_HANDLE,
            "local/blend_laplacian.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<TonemappingSettings>()
            .register_type::<TonemappingMode>()
            .register_type::<TonemappingCurve>();

        app.add_plugin(ExtractComponentPlugin::<TonemappingSettings>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<TonemappingPipeline>()
                .init_resource::<SpecializedRenderPipelines<TonemappingPipeline>>()
                .init_resource::<TonemappingLocalComputeLuminancesPipeline>()
                .init_resource::<SpecializedComputePipelines<TonemappingLocalComputeLuminancesPipeline>>()
                .init_resource::<TonemappingLocalComputeWeightsPipeline>()
                .init_resource::<SpecializedComputePipelines<TonemappingLocalComputeWeightsPipeline>>()
                .init_resource::<TonemappingLocalWeighExposuresPipeline>()
                .init_resource::<SpecializedComputePipelines<TonemappingLocalWeighExposuresPipeline>>()
                .init_resource::<TonemappingLocalBlendLaplacianPipeline>()
                .init_resource::<SpecializedComputePipelines<TonemappingLocalBlendLaplacianPipeline>>()
                .add_system_to_stage(RenderStage::Prepare, prepare_view_tonemapping_textures)
                .add_system_to_stage(RenderStage::Queue, queue_view_tonemapping_bind_groups)
                .add_system_to_stage(RenderStage::Queue, queue_view_tonemapping_pipelines);
        }
    }
}

#[derive(Resource)]
pub struct TonemappingPipeline {
    texture_bind_group: BindGroupLayout,
}

impl SpecializedRenderPipeline for TonemappingPipeline {
    type Key = TonemappingSettings;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();

        shader_defs.push(
            match key.curve {
                TonemappingCurve::ACESFilmic => "ACES_FILMIC",
                TonemappingCurve::Reinhard => "REINHARD",
                TonemappingCurve::ReinhardLuminance => "REINHARD_LUMINANCE",
            }
            .into(),
        );

        if key.deband_dither {
            shader_defs.push("DEBAND_DITHER".into());
        }

        RenderPipelineDescriptor {
            label: Some("tonemapping pipeline".into()),
            layout: Some(vec![self.texture_bind_group.clone()]),
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: TONEMAPPING_SHADER_HANDLE.typed(),
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: ViewTarget::TEXTURE_FORMAT_HDR,
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

impl FromWorld for TonemappingPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let tonemap_texture_bind_group = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("tonemapping_hdr_texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        TonemappingPipeline {
            texture_bind_group: tonemap_texture_bind_group,
        }
    }
}

#[derive(Resource)]
pub struct TonemappingLocalComputeLuminancesPipeline {
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for TonemappingLocalComputeLuminancesPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("tonemapping_local_compute_luminances_bind_group_layout"),
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
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        TonemappingLocalComputeLuminancesPipeline { bind_group_layout }
    }
}

impl SpecializedComputePipeline for TonemappingLocalComputeLuminancesPipeline {
    type Key = TonemappingSettings;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("tonemapping_local_compute_luminances_pipeline".into()),
            layout: Some(vec![self.bind_group_layout.clone()]),
            shader: TONEMAPPING_LOCAL_COMPUTE_LUMINANCES_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: "compute_luminances".into(),
        }
    }
}

#[derive(Resource)]
pub struct TonemappingLocalComputeWeightsPipeline {
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for TonemappingLocalComputeWeightsPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("tonemapping_local_compute_weights_bind_group_layout"),
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
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 3,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 4,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 5,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 6,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 7,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 8,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 9,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 10,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 11,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: ViewTarget::TEXTURE_FORMAT_HDR,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        TonemappingLocalComputeWeightsPipeline { bind_group_layout }
    }
}

impl SpecializedComputePipeline for TonemappingLocalComputeWeightsPipeline {
    type Key = TonemappingSettings;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("tonemapping_local_compute_weights_pipeline".into()),
            layout: Some(vec![self.bind_group_layout.clone()]),
            shader: TONEMAPPING_LOCAL_COMPUTE_WEIGHTS_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: "compute_weights".into(),
        }
    }
}

#[derive(Resource)]
pub struct TonemappingLocalWeighExposuresPipeline {
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for TonemappingLocalWeighExposuresPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("tonemapping_local_weigh_exposures_bind_group_layout"),
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
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: TextureFormat::R16Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        TonemappingLocalWeighExposuresPipeline { bind_group_layout }
    }
}

impl SpecializedComputePipeline for TonemappingLocalWeighExposuresPipeline {
    type Key = TonemappingSettings;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("tonemapping_local_weigh_exposures_pipeline".into()),
            layout: Some(vec![self.bind_group_layout.clone()]),
            shader: TONEMAPPING_LOCAL_WEIGH_EXPOSURES_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: "weigh_exposures".into(),
        }
    }
}

#[derive(Resource)]
pub struct TonemappingLocalBlendLaplacianPipeline {
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for TonemappingLocalBlendLaplacianPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("tonemapping_local_blend_laplacian_bind_group_layout"),
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
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: false },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 3,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 4,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: TextureFormat::R16Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 5,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        TonemappingLocalBlendLaplacianPipeline { bind_group_layout }
    }
}

impl SpecializedComputePipeline for TonemappingLocalBlendLaplacianPipeline {
    type Key = TonemappingSettings;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("tonemapping_local_blend_laplacian_pipeline".into()),
            layout: Some(vec![self.bind_group_layout.clone()]),
            shader: TONEMAPPING_LOCAL_BLEND_LAPLACIAN_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: "blend_laplacian".into(),
        }
    }
}

#[derive(Component)]
struct TonemappingLocalTextures {
    luminances: CachedTexture,
    weights: CachedTexture,
    assembly: CachedTexture,
}

impl TonemappingLocalTextures {
    pub fn texture_view(texture: &CachedTexture, base_mip_level: u32) -> TextureView {
        texture.texture.create_view(&TextureViewDescriptor {
            base_mip_level,
            mip_level_count: NonZeroU32::new(1),
            ..Default::default()
        })
    }
}

fn prepare_view_tonemapping_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera, &TonemappingSettings)>,
) {
    for (entity, camera, tonemapping_settings) in &views {
        if let (
            TonemappingMode::Local,
            Some(UVec2 {
                x: width,
                y: height,
            }),
        ) = (tonemapping_settings.mode, camera.physical_viewport_size)
        {
            let texture_descriptor = TextureDescriptor {
                label: None,
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 6,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: ViewTarget::TEXTURE_FORMAT_HDR,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            };

            let luminances = texture_cache.get(
                &render_device,
                TextureDescriptor {
                    label: Some("tonemapping_local_luminances"),
                    ..texture_descriptor
                },
            );

            let weights = texture_cache.get(
                &render_device,
                TextureDescriptor {
                    label: Some("tonemapping_local_weights"),
                    ..texture_descriptor
                },
            );

            let assembly = texture_cache.get(
                &render_device,
                TextureDescriptor {
                    label: Some("tonemapping_local_assembly"),
                    format: TextureFormat::R16Float,
                    ..texture_descriptor
                },
            );

            commands.entity(entity).insert(TonemappingLocalTextures {
                luminances,
                weights,
                assembly,
            });
        }
    }
}

#[derive(Component)]
pub struct TonemappingLocalBindGroups {
    compute_weights: BindGroup,
    weigh_exposures: BindGroup,
    blend_laplacians: [BindGroup; 5],
}

fn queue_view_tonemapping_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    local_compute_weights_pipeline: Res<TonemappingLocalComputeWeightsPipeline>,
    local_weigh_exposures_pipeline: Res<TonemappingLocalWeighExposuresPipeline>,
    local_blend_laplacians_pipeline: Res<TonemappingLocalBlendLaplacianPipeline>,
    views: Query<(Entity, &TonemappingLocalTextures)>,
) {
    let sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some("tonemapping_local_blend_laplacian_sampler"),
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    });

    for (entity, tonemapping_textures) in &views {
        let compute_weights = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("tonemapping_local_compute_weights_bind_group"),
            layout: &local_compute_weights_pipeline.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(
                            &tonemapping_textures.luminances,
                            0,
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(
                            &tonemapping_textures.luminances,
                            1,
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(
                            &tonemapping_textures.luminances,
                            2,
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(
                            &tonemapping_textures.luminances,
                            3,
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(
                            &tonemapping_textures.luminances,
                            4,
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(
                            &tonemapping_textures.luminances,
                            5,
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(&tonemapping_textures.weights, 0),
                    ),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(&tonemapping_textures.weights, 1),
                    ),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(&tonemapping_textures.weights, 2),
                    ),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(&tonemapping_textures.weights, 3),
                    ),
                },
                BindGroupEntry {
                    binding: 10,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(&tonemapping_textures.weights, 4),
                    ),
                },
                BindGroupEntry {
                    binding: 11,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(&tonemapping_textures.weights, 5),
                    ),
                },
            ],
        });

        let weigh_exposures = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("tonemapping_local_weigh_exposures_bind_group"),
            layout: &local_weigh_exposures_pipeline.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(
                            &tonemapping_textures.luminances,
                            5,
                        ),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(&tonemapping_textures.weights, 5),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &TonemappingLocalTextures::texture_view(&tonemapping_textures.assembly, 5),
                    ),
                },
            ],
        });

        let blend_laplacians = array::from_fn(|i| {
            render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("tonemapping_local_blend_laplacian_bind_group"),
                layout: &local_blend_laplacians_pipeline.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(
                            &TonemappingLocalTextures::texture_view(
                                &tonemapping_textures.luminances,
                                4 - i as u32,
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(
                            &TonemappingLocalTextures::texture_view(
                                &tonemapping_textures.luminances,
                                5 - i as u32,
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(
                            &TonemappingLocalTextures::texture_view(
                                &tonemapping_textures.weights,
                                4 - i as u32,
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(
                            &TonemappingLocalTextures::texture_view(
                                &tonemapping_textures.assembly,
                                5 - i as u32,
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: BindingResource::TextureView(
                            &TonemappingLocalTextures::texture_view(
                                &tonemapping_textures.assembly,
                                4 - i as u32,
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 5,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            })
        });

        commands.entity(entity).insert(TonemappingLocalBindGroups {
            compute_weights,
            weigh_exposures,
            blend_laplacians,
        });
    }
}

#[derive(Component)]
pub struct ViewTonemappingPipeline(CachedRenderPipelineId);

#[derive(Component)]
pub struct TonemappingLocalPipelineIds {
    compute_luminances: CachedComputePipelineId,
    compute_weights: CachedComputePipelineId,
    weigh_exposures: CachedComputePipelineId,
    blend_laplacian: CachedComputePipelineId,
}

pub fn queue_view_tonemapping_pipelines(
    mut commands: Commands,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<TonemappingPipeline>>,
    mut local_compute_luminances_pipelines: ResMut<
        SpecializedComputePipelines<TonemappingLocalComputeLuminancesPipeline>,
    >,
    local_compute_luminances_pipeline: Res<TonemappingLocalComputeLuminancesPipeline>,
    mut local_compute_weights_pipelines: ResMut<
        SpecializedComputePipelines<TonemappingLocalComputeWeightsPipeline>,
    >,
    local_compute_weights_pipeline: Res<TonemappingLocalComputeWeightsPipeline>,
    mut local_weigh_exposures_pipelines: ResMut<
        SpecializedComputePipelines<TonemappingLocalWeighExposuresPipeline>,
    >,
    local_weigh_exposures_pipeline: Res<TonemappingLocalWeighExposuresPipeline>,
    mut local_blend_laplacian_pipelines: ResMut<
        SpecializedComputePipelines<TonemappingLocalBlendLaplacianPipeline>,
    >,
    local_blend_laplacian_pipeline: Res<TonemappingLocalBlendLaplacianPipeline>,
    upscaling_pipeline: Res<TonemappingPipeline>,
    view_targets: Query<(Entity, &TonemappingSettings)>,
) {
    for (entity, tonemapping_settings) in view_targets.iter() {
        match tonemapping_settings.mode {
            TonemappingMode::Global => {
                let pipeline = pipelines.specialize(
                    &mut pipeline_cache,
                    &upscaling_pipeline,
                    tonemapping_settings.clone(),
                );

                commands
                    .entity(entity)
                    .insert(ViewTonemappingPipeline(pipeline));
            }

            TonemappingMode::Local => {
                let compute_luminances = local_compute_luminances_pipelines.specialize(
                    &mut pipeline_cache,
                    &local_compute_luminances_pipeline,
                    tonemapping_settings.clone(),
                );

                let compute_weights = local_compute_weights_pipelines.specialize(
                    &mut pipeline_cache,
                    &local_compute_weights_pipeline,
                    tonemapping_settings.clone(),
                );

                let weigh_exposures = local_weigh_exposures_pipelines.specialize(
                    &mut pipeline_cache,
                    &local_weigh_exposures_pipeline,
                    tonemapping_settings.clone(),
                );

                let blend_laplacian = local_blend_laplacian_pipelines.specialize(
                    &mut pipeline_cache,
                    &local_blend_laplacian_pipeline,
                    tonemapping_settings.clone(),
                );

                commands.entity(entity).insert(TonemappingLocalPipelineIds {
                    compute_luminances,
                    compute_weights,
                    weigh_exposures,
                    blend_laplacian,
                });
            }
        }
    }
}

#[derive(Component, Reflect, Clone, PartialEq, Eq, Hash)]
pub struct TonemappingSettings {
    pub mode: TonemappingMode,
    pub curve: TonemappingCurve,
    pub deband_dither: bool,
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TonemappingMode {
    Global,
    Local,
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TonemappingCurve {
    ACESFilmic,
    Reinhard,
    ReinhardLuminance,
}

impl ExtractComponent for TonemappingSettings {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = Self;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        Some(item.clone())
    }
}
