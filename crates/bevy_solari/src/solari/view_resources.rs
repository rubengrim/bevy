use super::camera::{PreviousViewProjection, PreviousViewProjectionUniforms, SolariSettings};
use bevy_core::FrameCount;
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_math::UVec2;
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
    g_buffer_previous: CachedTexture,
    g_buffer: CachedTexture,
    m_buffer: CachedTexture,
    t_buffer: CachedTexture,
    screen_probes_unfiltered: CachedTexture,
    screen_probes_filtered: CachedTexture,
    screen_probes_spherical_harmonics: CachedBuffer,
    indirect_diffuse: CachedTexture,
    indirect_diffuse_denoiser_temporal_history: CachedTexture,
    indirect_diffuse_denoised_temporal: CachedTexture,
    indirect_diffuse_denoised_spatiotemporal: CachedTexture,
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
    let texture = |label, format, size: UVec2| TextureDescriptor {
        label: Some(label),
        size: Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::STORAGE_BINDING,

        view_formats: &[],
    };
    let texture_double_buffered = |label_1, label_2, format, size: UVec2| {
        let shared = TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        };
        let t1 = TextureDescriptor {
            label: Some(label_1),
            ..shared
        };
        let t2 = TextureDescriptor {
            label: Some(label_2),
            ..shared
        };
        if frame_count.0 % 2 == 0 {
            (t1, t2)
        } else {
            (t2, t1)
        }
    };

    for (entity, camera) in &views {
        if let Some(viewport) = camera.physical_viewport_size {
            let (g_buffer_previous, g_buffer) = texture_double_buffered(
                "solari_g_buffer_1",
                "solari_g_buffer_2",
                TextureFormat::Rgba16Uint,
                viewport,
            );
            let m_buffer = texture("solari_m_buffer", TextureFormat::Rgba16Uint, viewport);
            let t_buffer = texture("solari_t_buffer", TextureFormat::Rg16Float, viewport);

            let width8 = round_up_to_multiple_of_8(viewport.x);
            let height8 = round_up_to_multiple_of_8(viewport.y);
            let size8 = UVec2::new(width8, height8);
            let probe_count = (width8 as u64 * height8 as u64) / 64;

            let screen_probes_unfiltered = texture(
                "solari_screen_probes_unfiltered",
                TextureFormat::Rgba32Float,
                size8,
            );
            let screen_probes_filtered = texture(
                "solari_screen_probes_filtered",
                TextureFormat::Rgba32Float,
                size8,
            );
            let screen_probes_spherical_harmonics = BufferDescriptor {
                label: Some("solari_screen_probes_spherical_harmonics"),
                size: probe_count * 112,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            };

            let indirect_diffuse = texture(
                "solari_indirect_diffuse",
                TextureFormat::Rgba16Float,
                viewport,
            );
            let (indirect_diffuse_denoiser_temporal_history, indirect_diffuse_denoised_temporal) =
                texture_double_buffered(
                    "solari_indirect_diffuse_temporal_denoise_1",
                    "solari_indirect_diffuse_temporal_denoise_2",
                    TextureFormat::Rgba16Float,
                    viewport,
                );
            let indirect_diffuse_denoised_spatiotemporal = texture(
                "solari_indirect_diffuse_denoised_spatiotemporal",
                TextureFormat::Rgba16Float,
                viewport,
            );

            let (taa_history_previous, taa_history_current) = texture_double_buffered(
                "solari_taa_history_1",
                "solari_taa_history_2",
                TextureFormat::Rgba16Float,
                viewport,
            );

            commands.entity(entity).insert(SolariResources {
                g_buffer_previous: texture_cache.get(&render_device, g_buffer_previous),
                g_buffer: texture_cache.get(&render_device, g_buffer),
                m_buffer: texture_cache.get(&render_device, m_buffer),
                t_buffer: texture_cache.get(&render_device, t_buffer),
                screen_probes_unfiltered: texture_cache
                    .get(&render_device, screen_probes_unfiltered),
                screen_probes_filtered: texture_cache.get(&render_device, screen_probes_filtered),
                screen_probes_spherical_harmonics: buffer_cache
                    .get(&render_device, screen_probes_spherical_harmonics),
                indirect_diffuse: texture_cache.get(&render_device, indirect_diffuse),
                indirect_diffuse_denoiser_temporal_history: texture_cache
                    .get(&render_device, indirect_diffuse_denoiser_temporal_history),
                indirect_diffuse_denoised_temporal: texture_cache
                    .get(&render_device, indirect_diffuse_denoised_temporal),
                indirect_diffuse_denoised_spatiotemporal: texture_cache
                    .get(&render_device, indirect_diffuse_denoised_spatiotemporal),
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
        let mut entry_i = 0;
        let mut entry = |ty| {
            entry_i += 1;
            BindGroupLayoutEntry {
                binding: entry_i - 1,
                visibility: ShaderStages::COMPUTE,
                ty,
                count: None,
            }
        };

        let entries = &[
            // View
            entry(BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: Some(ViewUniform::min_size()),
            }),
            // Previous view projection
            entry(BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: Some(PreviousViewProjection::min_size()),
            }),
            // G-buffer (previous)
            entry(BindingType::Texture {
                sample_type: TextureSampleType::Uint,
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            }),
            // G-buffer
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba16Uint,
                view_dimension: TextureViewDimension::D2,
            }),
            // M-buffer
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba16Uint,
                view_dimension: TextureViewDimension::D2,
            }),
            // T-buffer
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rg16Float,
                view_dimension: TextureViewDimension::D2,
            }),
            // Screen probes (unfiltered)
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba32Float,
                view_dimension: TextureViewDimension::D2,
            }),
            // Screen probes (filtered)
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba32Float,
                view_dimension: TextureViewDimension::D2,
            }),
            // Screen probe spherical harmonics
            entry(BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(112) }),
            }),
            // Indirect diffuse
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba16Float,
                view_dimension: TextureViewDimension::D2,
            }),
            // Indirect diffuse denoiser temporal history
            entry(BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: false },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            }),
            // Indirect diffuse denoised (temporal)
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba16Float,
                view_dimension: TextureViewDimension::D2,
            }),
            // Indirect diffuse denoised (spatiotemporal)
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba16Float,
                view_dimension: TextureViewDimension::D2,
            }),
            // TAA history (previous)
            entry(BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            }),
            // TAA history (current)
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                format: TextureFormat::Rgba16Float,
                view_dimension: TextureViewDimension::D2,
            }),
            // View target (other)
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba16Float,
                view_dimension: TextureViewDimension::D2,
            }),
            // View target (current)
            entry(BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                format: TextureFormat::Rgba16Float,
                view_dimension: TextureViewDimension::D2,
            }),
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
            let mut entry_i = 0;
            let mut entry = |resource| {
                entry_i += 1;
                BindGroupEntry {
                    binding: entry_i - 1,
                    resource,
                }
            };

            let entries = &[
                entry(view_uniforms.clone()),
                entry(previous_view_proj_uniforms.clone()),
                entry(t(&solari_resources.g_buffer_previous)),
                entry(t(&solari_resources.g_buffer)),
                entry(t(&solari_resources.m_buffer)),
                entry(t(&solari_resources.t_buffer)),
                entry(t(&solari_resources.screen_probes_unfiltered)),
                entry(t(&solari_resources.screen_probes_filtered)),
                entry(b(&solari_resources.screen_probes_spherical_harmonics)),
                entry(t(&solari_resources.indirect_diffuse)),
                entry(t(
                    &solari_resources.indirect_diffuse_denoiser_temporal_history
                )),
                entry(t(&solari_resources.indirect_diffuse_denoised_temporal)),
                entry(t(&solari_resources.indirect_diffuse_denoised_spatiotemporal)),
                entry(t(&solari_resources.taa_history_previous)),
                entry(t(&solari_resources.taa_history_current)),
                entry(BindingResource::TextureView(
                    view_target.main_texture_other_view(),
                )),
                entry(BindingResource::TextureView(
                    view_target.main_texture_view(),
                )),
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

fn t(texture: &CachedTexture) -> BindingResource<'_> {
    BindingResource::TextureView(&texture.default_view)
}

fn b(buffer: &CachedBuffer) -> BindingResource<'_> {
    buffer.buffer.as_entire_binding()
}
