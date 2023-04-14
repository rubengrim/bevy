use crate::{
    material::{GpuSolariMaterial, SolariMaterial},
    material_buffer::MaterialBuffer,
    pipeline::SolariPipeline,
    tlas::TlasResource,
};
use bevy_asset::{Assets, Handle};
use bevy_ecs::{
    prelude::Entity,
    system::{Commands, Query, Res, ResMut},
};
use bevy_render::{
    prelude::Mesh,
    render_resource::*,
    renderer::RenderDevice,
    view::{ViewTarget, ViewUniform, ViewUniforms},
    Extract,
};
use bevy_transform::prelude::GlobalTransform;

pub fn extract_meshes(
    meshes: Extract<
        Query<(
            Entity,
            &Handle<Mesh>,
            &Handle<SolariMaterial>,
            &GlobalTransform,
        )>,
    >,
    materials: Extract<Res<Assets<SolariMaterial>>>,
    mut material_buffer: ResMut<MaterialBuffer>,
    mut commands: Commands,
) {
    commands.insert_or_spawn_batch(
        meshes
            .iter()
            .filter_map(|(entity, mesh, material, transform)| {
                materials.get(material).map(|material| {
                    (
                        entity,
                        (
                            mesh.clone_weak(),
                            transform.clone(),
                            material_buffer.push(material),
                        ),
                    )
                })
            })
            .collect::<Vec<_>>(),
    );
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
                    min_binding_size: Some(GpuSolariMaterial::min_size()),
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

pub fn create_view_bind_group(
    view_target: &ViewTarget,
    view_uniforms: &ViewUniforms,
    tlas: &TlasResource,
    pipeline: &SolariPipeline,
    material_buffer: &MaterialBuffer,
    render_device: &RenderDevice,
) -> Option<BindGroup> {
    match (view_uniforms.uniforms.binding(), &tlas.0) {
        (Some(view_uniforms), Some(tlas)) => {
            Some(render_device.create_bind_group(&BindGroupDescriptor {
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
                        resource: BindingResource::TextureView(view_target.main_texture()),
                    },
                ],
            }))
        }
        _ => None,
    }
}
