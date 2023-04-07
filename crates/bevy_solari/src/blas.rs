use bevy_asset::Handle;
use bevy_ecs::system::{Query, Res};
use bevy_render::{
    mesh::{GpuBufferInfo, GpuMesh},
    prelude::Mesh,
    render_asset::RenderAssets,
    render_resource::{
        raytrace::{
            AccelerationStructureFlags, AccelerationStructureGeometryFlags,
            AccelerationStructureUpdateMode, BlasBuildEntry, BlasGeometries,
            BlasGeometrySizeDescriptors, BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor,
            CreateBlasDescriptor,
        },
        CommandEncoderDescriptor,
    },
    renderer::{RenderDevice, RenderQueue},
};
use std::ops::Deref;

pub fn prepare_blas(
    meshes: Query<&Handle<Mesh>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let blas_entries = meshes
        .iter()
        .filter_map(|mesh| render_meshes.get(mesh))
        .map(|gpu_mesh| todo!());

    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("blas_builder_command_encoder"),
    });

    unsafe { command_encoder.build_acceleration_structures_unsafe_tlas(blas_entries, &[]) };

    render_queue.submit([command_encoder.finish()]);
}

fn mesh_to_blas_entry<'a>(
    mesh: &Mesh,
    gpu_mesh: &GpuMesh,
    render_device: &RenderDevice,
) -> BlasBuildEntry<'a> {
    let (index_buffer, index_count, index_format, index_buffer_offset) = match &gpu_mesh.buffer_info
    {
        GpuBufferInfo::Indexed {
            buffer,
            count,
            index_format,
        } => (
            Some(buffer.deref()),
            Some(*count),
            Some(*index_format),
            Some(0),
        ),
        GpuBufferInfo::NonIndexed { .. } => (None, None, None, None),
    };

    let blas_size = BlasTriangleGeometrySizeDescriptor {
        vertex_format: Mesh::ATTRIBUTE_POSITION.format,
        vertex_count: mesh.count_vertices() as u32,
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

    BlasBuildEntry {
        blas: &blas,
        geometry: &BlasGeometries::TriangleGeometries(&[BlasTriangleGeometry {
            size: &blas_size,
            vertex_buffer: &gpu_mesh.vertex_buffer,
            first_vertex: 0,
            vertex_stride: todo!(),
            index_buffer,
            index_buffer_offset,
            transform_buffer: todo!(),
            transform_buffer_offset: todo!(),
        }]),
    }
}
