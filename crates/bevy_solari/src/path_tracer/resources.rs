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

#[derive(Component)]
pub struct SolariPathTracerAccumulationTexture(CachedTexture);

pub fn prepare_accumulation_textures(
    views: Query<(Entity, &ExtractedCamera), With<SolariPathTracer>>,
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
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

            commands
                .entity(entity)
                .insert(SolariPathTracerAccumulationTexture(
                    texture_cache.get(&render_device, texture_descriptor),
                ));
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
        ],
    })
}

pub fn create_view_bind_group(
    view_uniforms: &ViewUniforms,
    accumulation_texture: &SolariPathTracerAccumulationTexture,
    view_target: &ViewTarget,
    pipeline: &SolariPathtracerPipeline,
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
                    resource: BindingResource::TextureView(&accumulation_texture.0.default_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(view_target.main_texture_view()),
                },
            ],
        })
    })
}
