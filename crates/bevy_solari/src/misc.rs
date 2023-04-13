use crate::{
    material::{MaterialBuffer, SolariMaterial},
    pipeline::SolariPipeline,
    tlas::TlasResource,
};
use bevy_asset::Handle;
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
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

pub fn extract_meshes(
    meshes: Extract<Query<(Entity, &SolariMaterial, &GlobalTransform), With<Handle<Mesh>>>>,
    mut material_buffer: ResMut<MaterialBuffer>,
    mut commands: Commands,
) {
    commands.insert_or_spawn_batch(
        meshes
            .iter()
            .map(|(entity, material, transform)| {
                material_buffer.push(material.clone());
                (entity, transform.clone())
            })
            .collect::<Vec<_>>(),
    );
}

#[derive(Component)]
pub struct SolariTexture(pub CachedTexture);

pub fn prepare_textures(
    views: Query<(Entity, &ExtractedCamera)>,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    mut commands: Commands,
) {
    for (entity, camera) in &views {
        if let Some(viewport) = camera.physical_viewport_size {
            let descriptor = TextureDescriptor {
                label: Some("solari_output_texture"),
                size: Extent3d {
                    width: viewport.x,
                    height: viewport.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: ViewTarget::TEXTURE_FORMAT_HDR,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            commands
                .entity(entity)
                .insert(SolariTexture(texture_cache.get(&render_device, descriptor)));
        }
    }
}

pub fn create_view_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("view_bind_group_layout"),
        entries: &[
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
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::AccelerationStructure,
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(SolariMaterial::min_size()),
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
        ],
    })
}

#[derive(Component)]
pub struct ViewBindGroup(pub BindGroup);

pub fn queue_view_bind_group(
    views: Query<(Entity, &SolariTexture)>,
    view_uniforms: Res<ViewUniforms>,
    tlas: Res<TlasResource>,
    pipeline: Res<SolariPipeline>,
    material_buffer: Res<MaterialBuffer>,
    render_device: Res<RenderDevice>,
    mut commands: Commands,
) {
    if let (Some(view_uniforms), Some(tlas)) = (view_uniforms.uniforms.binding(), &tlas.0) {
        for (entity, SolariTexture(texture)) in &views {
            let descriptor = BindGroupDescriptor {
                label: Some("view_bind_group"),
                layout: &pipeline.view_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: view_uniforms.clone(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: tlas.as_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: material_buffer.binding(),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(&texture.default_view),
                    },
                ],
            };

            commands
                .entity(entity)
                .insert(ViewBindGroup(render_device.create_bind_group(&descriptor)));
        }
    }
}
