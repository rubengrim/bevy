use super::camera::{PreviousViewProjection, PreviousViewProjectionUniforms, SolariSettings};
use bevy_core::FrameCount;
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
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
pub struct SolariResources {
    g_buffer: CachedTexture,
    m_buffer: CachedTexture,
    t_buffer: CachedTexture,
    screen_probes_unfiltered: CachedTexture,
    screen_probes_filtered: CachedTexture,
    screen_probe_spherical_harmonics: CachedBuffer,
    taa_history_previous: CachedTexture,
    taa_history_current: CachedTexture,
}

pub fn prepare_resources(
    views: Query<(Entity, &ExtractedCamera), With<SolariSettings>>,
    frame_count: Res<FrameCount>,
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    mut buffer_cache: ResMut<BufferCache>,
    render_device: Res<RenderDevice>,
) {
    for (entity, camera) in &views {
        if let Some(viewport) = camera.physical_viewport_size {
            let g_buffer = TextureDescriptor {
                label: Some("solari_g_buffer"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: viewport.x,
                    height: viewport.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Uint,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            let m_buffer = TextureDescriptor {
                label: Some("solari_m_buffer"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: viewport.x,
                    height: viewport.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Uint,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            let t_buffer = TextureDescriptor {
                label: Some("solari_t_buffer"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: viewport.x,
                    height: viewport.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rg16Float,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            let width8 = round_up_to_multiple_of_8(viewport.x);
            let height8 = round_up_to_multiple_of_8(viewport.y);
            let probe_count = (width8 as u64 * height8 as u64) / 64;

            let screen_probes_unfiltered = TextureDescriptor {
                label: Some("solari_screen_probes_unfiltered"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: width8,
                    height: height8,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };
            let screen_probes_filtered = TextureDescriptor {
                label: Some("solari_screen_probes_filtered"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: width8,
                    height: height8,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            let screen_probe_spherical_harmonics = BufferDescriptor {
                label: Some("solari_screen_probe_spherical_harmonics"),
                size: probe_count * 112,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let taa_history_1 = TextureDescriptor {
                label: Some("solari_taa_history_1"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: viewport.x,
                    height: viewport.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };
            let taa_history_2 = TextureDescriptor {
                label: Some("solari_taa_history_2"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: viewport.x,
                    height: viewport.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };
            let (taa_history_previous, taa_history_current) = if frame_count.0 % 2 == 0 {
                (taa_history_1, taa_history_2)
            } else {
                (taa_history_2, taa_history_1)
            };

            commands.entity(entity).insert(SolariResources {
                g_buffer: texture_cache.get(&render_device, g_buffer),
                m_buffer: texture_cache.get(&render_device, m_buffer),
                t_buffer: texture_cache.get(&render_device, t_buffer),
                screen_probes_unfiltered: texture_cache
                    .get(&render_device, screen_probes_unfiltered),
                screen_probes_filtered: texture_cache.get(&render_device, screen_probes_filtered),
                screen_probe_spherical_harmonics: buffer_cache
                    .get(&render_device, screen_probe_spherical_harmonics),
                taa_history_previous: texture_cache.get(&render_device, taa_history_previous),
                taa_history_current: texture_cache.get(&render_device, taa_history_current),
            });
        }
    }
}

#[derive(Resource)]
pub struct SolariBindGroupLayout(pub BindGroupLayout);

impl FromWorld for SolariBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let entries = &[
            // View
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
            // Previous view projection
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(PreviousViewProjection::min_size()),
                },
                count: None,
            },
            // G-buffer
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: TextureFormat::Rgba16Uint,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // M-buffer
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: TextureFormat::Rgba16Uint,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // T-buffer
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: TextureFormat::Rg16Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // Screen probes (unfiltered)
            BindGroupLayoutEntry {
                binding: 5,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: TextureFormat::Rgba32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // Screen probes (filtered)
            BindGroupLayoutEntry {
                binding: 6,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: TextureFormat::Rgba32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // Screen probe spherical harmonics
            BindGroupLayoutEntry {
                binding: 7,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(112) }),
                },
                count: None,
            },
            // TAA history (previous)
            BindGroupLayoutEntry {
                binding: 8,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // TAA history (current)
            BindGroupLayoutEntry {
                binding: 9,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::Rgba16Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // View target (other)
            BindGroupLayoutEntry {
                binding: 10,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: TextureFormat::Rgba16Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // View target (current)
            BindGroupLayoutEntry {
                binding: 11,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::Rgba16Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
        ];

        Self(world.resource::<RenderDevice>().create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("solari_bind_group_layout"),
                entries,
            },
        ))
    }
}

#[derive(Component)]
pub struct SolariBindGroup(pub BindGroup);

pub fn queue_bind_groups(
    views: Query<(Entity, &SolariResources, &ViewTarget)>,
    view_uniforms: Res<ViewUniforms>,
    previous_view_proj_uniforms: Res<PreviousViewProjectionUniforms>,
    bind_group_layout: Res<SolariBindGroupLayout>,
    mut commands: Commands,
    render_device: Res<RenderDevice>,
) {
    if let (Some(view_uniforms), Some(previous_view_proj_uniforms)) = (
        view_uniforms.uniforms.binding(),
        previous_view_proj_uniforms.uniforms.binding(),
    ) {
        for (entity, solari_resources, view_target) in &views {
            let entries = &[
                BindGroupEntry {
                    binding: 0,
                    resource: view_uniforms.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: previous_view_proj_uniforms.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&solari_resources.g_buffer.default_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&solari_resources.m_buffer.default_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&solari_resources.t_buffer.default_view),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(
                        &solari_resources.screen_probes_unfiltered.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::TextureView(
                        &solari_resources.screen_probes_filtered.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: solari_resources
                        .screen_probe_spherical_harmonics
                        .buffer
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::TextureView(
                        &solari_resources.taa_history_previous.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::TextureView(
                        &solari_resources.taa_history_current.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 10,
                    resource: BindingResource::TextureView(view_target.main_texture_other()),
                },
                BindGroupEntry {
                    binding: 11,
                    resource: BindingResource::TextureView(view_target.main_texture()),
                },
            ];

            commands
                .entity(entity)
                .insert(SolariBindGroup(render_device.create_bind_group(
                    &BindGroupDescriptor {
                        label: Some("solari_bind_group"),
                        layout: &bind_group_layout.0,
                        entries,
                    },
                )));
        }
    }
}

fn round_up_to_multiple_of_8(x: u32) -> u32 {
    (x + 7) & !7
}
