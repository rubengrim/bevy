use std::num::NonZeroU64;

use super::camera::SolariSettings;
use bevy_core::FrameCount;
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::{
    camera::ExtractedCamera,
    globals::{GlobalsBuffer, GlobalsUniform},
    render_resource::*,
    renderer::RenderDevice,
    texture::{CachedTexture, TextureCache},
    view::{ViewTarget, ViewUniform, ViewUniforms},
};

#[derive(Component)]
pub struct SolariResources {
    g_buffer: CachedTexture,
    m_buffer: CachedTexture,
    screen_probes_unfiltered: CachedTexture,
    screen_probes_filtered: CachedTexture,
    screen_probe_spherical_harmonics: Buffer,
}

pub fn prepare_resources(
    views: Query<(Entity, &ExtractedCamera), With<SolariSettings>>,
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    frame_count: Res<FrameCount>,
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

            let width8 = round_up_to_multiple_of_8(viewport.x);
            let height8 = round_up_to_multiple_of_8(viewport.y);
            let probe_count = (width8 as u64 * height8 as u64) / 64;

            let screen_probes_a = TextureDescriptor {
                label: Some("solari_screen_probes_a"),
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
            let screen_probes_b = TextureDescriptor {
                label: Some("solari_screen_probes_b"),
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
            let (screen_probes_unfiltered, screen_probes_filtered) = match frame_count.0 % 2 == 0 {
                true => (screen_probes_a, screen_probes_b),
                false => (screen_probes_b, screen_probes_a),
            };

            // TODO: Cache buffer
            let screen_probe_spherical_harmonics = BufferDescriptor {
                label: Some("solari_screen_probe_spherical_harmonics"),
                size: probe_count * 112,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            commands.entity(entity).insert(SolariResources {
                g_buffer: texture_cache.get(&render_device, g_buffer),
                m_buffer: texture_cache.get(&render_device, m_buffer),
                screen_probes_unfiltered: texture_cache
                    .get(&render_device, screen_probes_unfiltered),
                screen_probes_filtered: texture_cache.get(&render_device, screen_probes_filtered),
                screen_probe_spherical_harmonics: render_device
                    .create_buffer(&screen_probe_spherical_harmonics),
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
            // Globals
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(GlobalsUniform::min_size()),
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
            // Screen probes (unfiltered)
            BindGroupLayoutEntry {
                binding: 4,
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
                binding: 5,
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
                binding: 6,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(112) }),
                },
                count: None,
            },
            // View target
            BindGroupLayoutEntry {
                binding: 7,
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
    globals: Res<GlobalsBuffer>,
    bind_group_layout: Res<SolariBindGroupLayout>,
    mut commands: Commands,
    render_device: Res<RenderDevice>,
) {
    if let (Some(view_uniforms), Some(globals)) =
        (view_uniforms.uniforms.binding(), globals.buffer.binding())
    {
        for (entity, solari_resources, view_target) in &views {
            let entries = &[
                BindGroupEntry {
                    binding: 0,
                    resource: view_uniforms.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: globals.clone(),
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
                    resource: BindingResource::TextureView(
                        &solari_resources.screen_probes_unfiltered.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(
                        &solari_resources.screen_probes_filtered.default_view,
                    ),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: solari_resources
                        .screen_probe_spherical_harmonics
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 7,
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
