use bevy_asset::Handle;
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use bevy_render::{
    mesh::GpuBufferInfo,
    prelude::Mesh,
    render_asset::RenderAssets,
    render_resource::{
        raytrace::*, Buffer, CommandEncoderDescriptor, IndexFormat, PrimitiveTopology,
    },
    renderer::{RenderDevice, RenderQueue},
};
use bevy_utils::HashMap;
use std::{mem, ops::Deref};

#[derive(Resource, Default)]
pub struct BlasStorage {
    storage: HashMap<Handle<Mesh>, Blas>,
}

impl BlasStorage {
    pub fn get(&self, mesh: &Handle<Mesh>) -> Option<&Blas> {
        self.storage.get(mesh)
    }
}

pub fn prepare_blas(
    meshes: Query<&Handle<Mesh>>,
    mut blas_storage: ResMut<BlasStorage>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let mut blas_build_queue = Vec::new();
    let mut blas_sizes = Vec::new();
    for mesh in &meshes {
        if let (Some(gpu_mesh), None) = (render_meshes.get(mesh), blas_storage.get(mesh)) {
            if gpu_mesh.primitive_topology != PrimitiveTopology::TriangleList {
                continue;
            }

            let (index_buffer, index_count, index_format, index_buffer_offset, vertex_count) =
                map_buffer_info(&gpu_mesh.buffer_info);

            let blas_size = BlasTriangleGeometrySizeDescriptor {
                vertex_format: Mesh::ATTRIBUTE_POSITION.format,
                vertex_count,
                index_format,
                index_count,
                flags: AccelerationStructureGeometryFlags::OPAQUE,
            };

            let blas = render_device.wgpu_device().create_blas(
                &CreateBlasDescriptor {
                    label: None,
                    flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
                    update_mode: AccelerationStructureUpdateMode::Build,
                },
                BlasGeometrySizeDescriptors::Triangles {
                    desc: vec![blas_size.clone()],
                },
            );

            blas_sizes.push(blas_size);

            let blas_geometries = [BlasTriangleGeometry {
                size: blas_sizes.last().unwrap(),
                vertex_buffer: &gpu_mesh.vertex_buffer,
                first_vertex: 0,
                vertex_stride: gpu_mesh.layout.layout().array_stride,
                index_buffer: index_buffer.map(Deref::deref),
                index_buffer_offset,
                transform_buffer: None,
                transform_buffer_offset: None,
            }];

            let blas_entry = blas_storage.storage.entry(mesh.clone_weak()).insert(blas);

            let blas_build_entry = BlasBuildEntry {
                blas: blas_entry.get(),
                geometry: &BlasGeometries::TriangleGeometries(&blas_geometries),
            };

            blas_build_queue.push(unsafe { mem::transmute(blas_build_entry) });
        }
    }

    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("prepare_blas_command_encoder"),
    });
    unsafe { command_encoder.build_acceleration_structures_unsafe_tlas(&blas_build_queue, &[]) };
    render_queue.submit([command_encoder.finish()]);
}

fn map_buffer_info(
    buffer_info: &GpuBufferInfo,
) -> (
    Option<&Buffer>,
    Option<u32>,
    Option<IndexFormat>,
    Option<u64>,
    u32,
) {
    match buffer_info {
        GpuBufferInfo::Indexed {
            buffer,
            count,
            index_format,
            vertex_count,
        } => (
            Some(buffer),
            Some(*count),
            Some(*index_format),
            Some(0),
            *vertex_count,
        ),
        GpuBufferInfo::NonIndexed { vertex_count } => (None, None, None, None, *vertex_count),
    }
}
