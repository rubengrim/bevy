use super::{
    bind_group_layout::SolariSceneResources,
    blas::BlasStorage,
    material::{GpuSolariMaterial, MeshMaterial, SolariMaterial},
    misc::{new_storage_buffer, tlas_transform, IndexedVec},
};
use bevy_asset::Handle;
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use bevy_render::{
    mesh::GpuBufferInfo,
    prelude::{Color, Mesh},
    render_asset::RenderAssets,
    render_resource::{raytrace::*, *},
    renderer::{RenderDevice, RenderQueue},
    texture::{FallbackImage, Image},
};
use bevy_transform::prelude::GlobalTransform;
use std::iter;

#[derive(Resource, Default)]
pub struct SolariSceneBindGroup(pub Option<BindGroup>);

pub fn queue_scene_bind_group(
    objects: Query<(
        &Handle<Mesh>,
        &Handle<SolariMaterial>,
        &SolariMaterial,
        &GlobalTransform,
    )>,
    mut scene_bind_group: ResMut<SolariSceneBindGroup>,
    scene_resources: Res<SolariSceneResources>,
    mesh_assets: Res<RenderAssets<Mesh>>,
    image_assets: Res<RenderAssets<Image>>,
    blas_storage: Res<BlasStorage>,
    fallback_image: Res<FallbackImage>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    // Create CPU buffers for scene resources
    // TODO: Reuse memory each frame
    let mut mesh_materials = IndexedVec::new();
    let mut index_buffers = IndexedVec::new();
    let mut vertex_buffers = Vec::new();
    let mut materials = IndexedVec::new();
    let mut texture_maps = IndexedVec::new();
    let objects = objects.iter().collect::<Vec<_>>();

    let mut get_mesh_index = |mesh_handle| {
        index_buffers.get_index(mesh_handle, |mesh_handle| {
            let gpu_mesh = mesh_assets.get(mesh_handle).unwrap();
            vertex_buffers.push(gpu_mesh.vertex_buffer.as_entire_buffer_binding());
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed { buffer, .. } => buffer.as_entire_buffer_binding(),
                _ => unreachable!(), // TODO: Handle non-indexed meshes
            }
        })
    };

    let mut get_texture_map_index = |maybe_texture_map_handle: &Option<Handle<Image>>| {
        if let Some(texture_map_handle) = maybe_texture_map_handle.clone() {
            texture_maps.get_index(texture_map_handle, |texture_map_handle| {
                // TODO: Handle unwrap
                &*image_assets.get(&texture_map_handle).unwrap().texture_view
            })
        } else {
            u32::MAX
        }
    };

    let mut get_material_index = |material_handle, material: &SolariMaterial| {
        let emission = material
            .emission
            .unwrap_or(Color::BLACK)
            .as_linear_rgba_f32();
        materials.get_index(material_handle, |_| GpuSolariMaterial {
            base_color: material.base_color.as_linear_rgba_f32().into(),
            base_color_map_index: get_texture_map_index(&material.base_color_map),
            emission: [emission[0], emission[1], emission[2]].into(),
        })
    };

    // Create TLAS
    let mut tlas = TlasPackage::new(
        render_device
            .wgpu_device()
            .create_tlas(&CreateTlasDescriptor {
                label: Some("tlas"),
                flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
                update_mode: AccelerationStructureUpdateMode::Build,
                max_instances: objects.len() as u32,
            }),
        objects.len() as u32,
    );

    // Fill TLAS and scene buffers
    // TODO: Parallelize loop
    for (i, (mesh_handle, material_handle, material, transform)) in objects.into_iter().enumerate()
    {
        if let Some(blas) = blas_storage.get(mesh_handle) {
            let object_i = mesh_materials.get_index(
                (mesh_handle, material_handle),
                |(mesh_handle, material_handle)| MeshMaterial {
                    mesh_index: get_mesh_index(mesh_handle),
                    material_index: get_material_index(material_handle, material),
                },
            );

            *tlas.get_mut_single(i).unwrap() = Some(TlasInstance::new(
                blas,
                tlas_transform(transform),
                object_i,
                0xFF,
            ));
        }
    }

    // Build TLAS
    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("build_tlas_command_encoder"),
    });
    command_encoder.build_acceleration_structures(&[], iter::once(&tlas));
    render_queue.submit([command_encoder.finish()]);

    // Upload buffers to the GPU
    // TODO: Reuse GPU buffers each frame
    let mesh_materials_buffer = new_storage_buffer(
        mesh_materials.vec,
        "solari_mesh_materials_buffer",
        &render_device,
        &render_queue,
    );
    let materials_buffer = new_storage_buffer(
        materials.vec,
        "solari_material_buffer",
        &render_device,
        &render_queue,
    );

    // Ensure binding arrays are non-empty
    if vertex_buffers.is_empty() {
        return;
    }
    if texture_maps.vec.is_empty() {
        texture_maps.vec.push(&fallback_image.texture_view);
    }

    // Create scene bind group
    scene_bind_group.0 = Some(render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("solari_scene_bind_group"),
        layout: &scene_resources.bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::AccelerationStructure(tlas.tlas()),
            },
            BindGroupEntry {
                binding: 1,
                resource: mesh_materials_buffer.binding().unwrap(),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::BufferArray(index_buffers.vec.as_slice()),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::BufferArray(vertex_buffers.as_slice()),
            },
            BindGroupEntry {
                binding: 4,
                resource: materials_buffer.binding().unwrap(),
            },
            BindGroupEntry {
                binding: 5,
                resource: BindingResource::TextureViewArray(texture_maps.vec.as_slice()),
            },
            BindGroupEntry {
                binding: 6,
                resource: BindingResource::Sampler(&scene_resources.sampler),
            },
        ],
    }));
}