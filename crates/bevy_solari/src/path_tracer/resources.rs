use super::{camera::SolariPathTracer, pipeline::SolariPathtracerPipeline};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use bevy_render::{
    camera::ExtractedCamera,
    render_resource::*,
    renderer::RenderDevice,
    texture::{CachedTexture, TextureCache},
    view::{ViewTarget, ViewUniform, ViewUniforms},
};
use std::num::NonZeroU64;

#[derive(Component)]
pub struct SolariPathTracerAccumulationTexture {
    accumulation_texture: CachedTexture,
    rays: CachedBuffer,
    keys32: CachedBuffer,
    block_start_for_radix: CachedBuffer,
    is_ordered: CachedBuffer,
    starting_bit: CachedBuffer,
    scan_results: CachedBuffer,
    prefix_sum_array: CachedBuffer,
    rays_output: CachedBuffer,
    radix_block_info: CachedBuffer,
    block_start_for_radix_output: CachedBuffer,
    keys32_out: CachedBuffer,
    id_of_id: CachedBuffer,
    block_sums: CachedBuffer,
}

pub fn prepare_accumulation_textures(
    views: Query<(Entity, &ExtractedCamera), With<SolariPathTracer>>,
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    mut buffer_cache: ResMut<BufferCache>,
    render_device: Res<RenderDevice>,
) {
    for (entity, camera) in &views {
        if let Some(viewport) = camera.physical_viewport_size {
            let texture_descriptor = TextureDescriptor {
                label: Some("solari_path_tracer_accumulation_texture"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: viewport.x,
                    height: viewport.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            let ray_count = (((viewport.x * viewport.y) + 63) / 64) as u64;
            let block_count = (ray_count / 64) as u64;

            let rays = BufferDescriptor {
                label: Some("rays"),
                size: ray_count * 32,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let keys32 = BufferDescriptor {
                label: Some("keys32"),
                size: ray_count * 4,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let block_start_for_radix = BufferDescriptor {
                label: Some("block_start_for_radix"),
                size: ray_count * 4,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let is_ordered = BufferDescriptor {
                label: Some("is_ordered"),
                size: 4,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let starting_bit = BufferDescriptor {
                label: Some("starting_bit"),
                size: 4,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let scan_results = BufferDescriptor {
                label: Some("scan_results"),
                size: block_count * 16,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let prefix_sum_array = BufferDescriptor {
                label: Some("prefix_sum_array"),
                size: ray_count * 16,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let rays_output = BufferDescriptor {
                label: Some("rays_output"),
                size: ray_count * 32,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let radix_block_info = BufferDescriptor {
                label: Some("radix_block_info"),
                size: ray_count * 16,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let block_start_for_radix_output = BufferDescriptor {
                label: Some("block_start_for_radix_output"),
                size: ray_count * 4,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let keys32_out = BufferDescriptor {
                label: None,
                size: ray_count * 4,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };
            let id_of_id = BufferDescriptor {
                label: Some("id_of_id"),
                size: ray_count * 4,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let block_sums = BufferDescriptor {
                label: Some("block_sums"),
                size: block_count * 16,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            commands
                .entity(entity)
                .insert(SolariPathTracerAccumulationTexture {
                    accumulation_texture: texture_cache.get(&render_device, texture_descriptor),
                    rays: buffer_cache.get(&render_device, rays),
                    keys32: buffer_cache.get(&render_device, keys32),
                    block_start_for_radix: buffer_cache.get(&render_device, block_start_for_radix),
                    is_ordered: buffer_cache.get(&render_device, is_ordered),
                    starting_bit: buffer_cache.get(&render_device, starting_bit),
                    scan_results: buffer_cache.get(&render_device, scan_results),
                    prefix_sum_array: buffer_cache.get(&render_device, prefix_sum_array),
                    rays_output: buffer_cache.get(&render_device, rays_output),
                    radix_block_info: buffer_cache.get(&render_device, radix_block_info),
                    block_start_for_radix_output: buffer_cache
                        .get(&render_device, block_start_for_radix_output),
                    keys32_out: buffer_cache.get(&render_device, keys32_out),
                    id_of_id: buffer_cache.get(&render_device, id_of_id),
                    block_sums: buffer_cache.get(&render_device, block_sums),
                });
        }
    }
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
            // Accumulation texture
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: TextureFormat::Rgba32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // Output texture
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: ViewTarget::TEXTURE_FORMAT_HDR,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // Rays
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(32) }),
                },
                count: None,
            },
            // Keys32
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(4) }),
                },
                count: None,
            },
            // Block start for radix
            BindGroupLayoutEntry {
                binding: 5,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(4) }),
                },
                count: None,
            },
            // Is ordered
            BindGroupLayoutEntry {
                binding: 6,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(4) }),
                },
                count: None,
            },
            // Starting bit
            BindGroupLayoutEntry {
                binding: 7,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(4) }),
                },
                count: None,
            },
            // Scan results
            BindGroupLayoutEntry {
                binding: 8,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(16) }),
                },
                count: None,
            },
            // Prefix sum array
            BindGroupLayoutEntry {
                binding: 9,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(16) }),
                },
                count: None,
            },
            // Rays output
            BindGroupLayoutEntry {
                binding: 10,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(32) }),
                },
                count: None,
            },
            // Radix block info
            BindGroupLayoutEntry {
                binding: 11,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(16) }),
                },
                count: None,
            },
            // Block start for radix output
            BindGroupLayoutEntry {
                binding: 12,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(4) }),
                },
                count: None,
            },
            // Key32s out
            BindGroupLayoutEntry {
                binding: 13,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(4) }),
                },
                count: None,
            },
            // Id of id
            BindGroupLayoutEntry {
                binding: 14,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(4) }),
                },
                count: None,
            },
            // Block sums
            BindGroupLayoutEntry {
                binding: 15,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(16) }),
                },
                count: None,
            },
        ],
    })
}

pub fn create_view_bind_group(
    view_uniforms: &ViewUniforms,
    accumulation_texture: &SolariPathTracerAccumulationTexture,
    view_target: &ViewTarget,
    pipeline: &SolariPathtracerPipeline,
    render_device: &RenderDevice,
    swap: bool,
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
                        &accumulation_texture.accumulation_texture.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(view_target.main_texture()),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: if swap {
                        accumulation_texture.rays.buffer.as_entire_binding()
                    } else {
                        accumulation_texture.rays_output.buffer.as_entire_binding()
                    },
                },
                BindGroupEntry {
                    binding: 4,
                    resource: accumulation_texture.keys32.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: accumulation_texture
                        .block_start_for_radix
                        .buffer
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: accumulation_texture.is_ordered.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: accumulation_texture.starting_bit.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: accumulation_texture.scan_results.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: accumulation_texture
                        .prefix_sum_array
                        .buffer
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 10,
                    resource: if swap {
                        accumulation_texture.rays.buffer.as_entire_binding()
                    } else {
                        accumulation_texture.rays_output.buffer.as_entire_binding()
                    },
                },
                BindGroupEntry {
                    binding: 11,
                    resource: accumulation_texture
                        .radix_block_info
                        .buffer
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 12,
                    resource: accumulation_texture
                        .block_start_for_radix_output
                        .buffer
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 13,
                    resource: accumulation_texture.keys32_out.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 14,
                    resource: accumulation_texture.id_of_id.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 15,
                    resource: accumulation_texture.block_sums.buffer.as_entire_binding(),
                },
            ],
        })
    })
}
