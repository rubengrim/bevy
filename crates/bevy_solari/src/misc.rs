use crate::{
    material::{GpuSolariMaterial, SolariMaterial},
    pipeline::SolariPipeline,
    scene::MeshMaterial,
};
use bevy_asset::{Assets, Handle};
use bevy_ecs::{
    prelude::{Component, Entity},
    system::{Commands, Query, Res, ResMut},
};
use bevy_render::{
    camera::ExtractedCamera,
    prelude::Mesh,
    render_resource::*,
    renderer::RenderDevice,
    texture::{CachedTexture, TextureCache},
    view::{ViewTarget, ViewUniform, ViewUniforms},
    Extract,
};
use bevy_transform::prelude::GlobalTransform;
use std::ops::{Div, Sub};

pub fn extract_objects(
    meshes: Extract<
        Query<(
            Entity,
            &Handle<Mesh>,
            &Handle<SolariMaterial>,
            &GlobalTransform,
        )>,
    >,
    materials: Extract<Res<Assets<SolariMaterial>>>,
    mut commands: Commands,
) {
    commands.insert_or_spawn_batch(
        meshes
            .iter()
            .filter_map(|(entity, mesh_handle, material_handle, transform)| {
                materials.get(material_handle).map(|material| {
                    (
                        entity,
                        (
                            mesh_handle.clone_weak(),
                            material_handle.clone_weak(),
                            material.clone(),
                            transform.clone(),
                        ),
                    )
                })
            })
            .collect::<Vec<_>>(),
    );
}

#[derive(Component)]
pub struct SolariTextures {
    pub screen_probes: CachedTexture,
}

pub fn prepare_textures(
    views: Query<(Entity, &ExtractedCamera)>,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    mut commands: Commands,
) {
    for (entity, camera) in &views {
        if let Some(viewport) = camera.physical_viewport_size {
            let screen_probes_descriptor = TextureDescriptor {
                label: Some("solari_screen_probes"),
                size: Extent3d {
                    width: round_up_to_multiple_of_8(viewport.x),
                    height: round_up_to_multiple_of_8(viewport.y),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: ViewTarget::TEXTURE_FORMAT_HDR,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            commands.entity(entity).insert(SolariTextures {
                screen_probes: texture_cache.get(&render_device, screen_probes_descriptor),
            });
        }
    }
}

pub fn create_scene_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    // https://vulkan.gpuinfo.org/displaydevicelimit.php?name=maxPerStageDescriptorStorageBuffers
    let max_buffer_bindings = Some(
        render_device
            .limits()
            .max_storage_buffers_per_shader_stage
            .div(2)
            .min(65000000 / 2)
            .sub(10000)
            .try_into()
            .unwrap(),
    );
    // https://vulkan.gpuinfo.org/displaydevicelimit.php?name=maxPerStageDescriptorSampledImages
    let max_texture_map_bindings = Some(
        render_device
            .limits()
            .max_sampled_textures_per_shader_stage
            .min(65000000)
            .sub(10000)
            .try_into()
            .unwrap(),
    );

    let entries = &[
        // TLAS
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::AccelerationStructure,
            count: None,
        },
        // MeshMaterial buffer
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(MeshMaterial::min_size()),
            },
            count: None,
        },
        // Index buffers
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None, // TODO
            },
            count: max_buffer_bindings,
        },
        // Vertex buffers
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None, // TODO
            },
            count: max_buffer_bindings,
        },
        // Material buffer
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(GpuSolariMaterial::min_size()),
            },
            count: None,
        },
        // Texture maps
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: max_texture_map_bindings,
        },
        // Texture sampler
        BindGroupLayoutEntry {
            binding: 6,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        },
    ];

    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("solari_scene_bind_group_layout"),
        entries,
    })
}

pub fn create_view_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("solari_view_bind_group_layout"),
        entries: &[
            // View uniforms
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            },
            // Screen probes
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: ViewTarget::TEXTURE_FORMAT_HDR,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // Output texture
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
        ],
    })
}

pub fn create_view_bind_group(
    view_uniforms: &ViewUniforms,
    view_target: &ViewTarget,
    solari_textures: &SolariTextures,
    pipeline: &SolariPipeline,
    render_device: &RenderDevice,
) -> Option<BindGroup> {
    view_uniforms.uniforms.binding().map(|view_uniforms| {
        render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("solari_view_bind_group"),
            layout: &pipeline.view_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: view_uniforms.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &solari_textures.screen_probes.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(view_target.main_texture()),
                },
            ],
        })
    })
}

fn round_up_to_multiple_of_8(x: u32) -> u32 {
    (x + 7) & !7
}
