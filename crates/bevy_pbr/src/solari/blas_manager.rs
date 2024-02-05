use bevy_asset::{AssetEvent, AssetId, Handle};
use bevy_ecs::{
    event::EventReader,
    system::{Res, ResMut, Resource, SystemState},
    world::{FromWorld, Mut, World},
};
use bevy_render::{
    mesh::{GpuBufferInfo, GpuMesh, Mesh},
    render_asset::RenderAssets,
    render_resource::{
        ray_tracing::*, Buffer, CommandEncoderDescriptor, IndexFormat, PrimitiveTopology,
    },
    renderer::{RenderDevice, RenderQueue},
    MainWorld,
};
use bevy_utils::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct BlasManager {
    blas: HashMap<AssetId<Mesh>, Blas>,
    changed: HashSet<AssetId<Mesh>>,
    removed: Vec<AssetId<Mesh>>,
}

impl BlasManager {
    pub fn get(&self, mesh: &Handle<Mesh>) -> Option<&Blas> {
        self.blas.get(&mesh.id())
    }
}

#[derive(Resource)]
pub struct ExtractMeshAssetEventsSystemState {
    state: SystemState<EventReader<'static, 'static, AssetEvent<Mesh>>>,
}

impl FromWorld for ExtractMeshAssetEventsSystemState {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: SystemState::new(world),
        }
    }
}

pub fn extract_mesh_asset_events(
    mut main_world: ResMut<MainWorld>,
    mut blas_manager: ResMut<BlasManager>,
) {
    main_world.resource_scope(
        |main_world, mut state: Mut<ExtractMeshAssetEventsSystemState>| {
            for asset_event in state.state.get(main_world).read() {
                match asset_event {
                    AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                        blas_manager.changed.insert(*id);
                    }
                    AssetEvent::Unused { id } => {
                        blas_manager.removed.push(*id);
                        blas_manager.changed.remove(id);
                    }
                    _ => {}
                }
            }
        },
    );
}

// TODO: BLAS compaction
// TODO: Async compute queue for BLAS creation
pub fn prepare_new_blas(
    mut blas_manager: ResMut<BlasManager>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let blas_manager = blas_manager.as_mut();

    // Delete BLAS for removed meshes
    for asset_id in blas_manager.removed.drain(..) {
        blas_manager.blas.remove(&asset_id);
    }

    if blas_manager.changed.is_empty() {
        return;
    }

    // Get GpuMeshes and filter to solari-compatible meshes
    let meshes = blas_manager
        .changed
        .drain()
        .filter_map(|asset_id| match render_meshes.get(asset_id) {
            Some(gpu_mesh) if mesh_compatible(gpu_mesh) => Some((asset_id, gpu_mesh)),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Create BLAS, blas size for each mesh
    let blas_resources = meshes
        .iter()
        .map(|(asset_id, gpu_mesh)| setup_blas(asset_id, gpu_mesh, blas_manager, &render_device))
        .collect::<Vec<_>>();

    // Create list of BlasBuildEntries using blas_resources
    let build_entries = blas_resources
        .iter()
        .map(
            |(asset_id, gpu_mesh, blas_size, index_buffer)| BlasBuildEntry {
                blas: blas_manager.blas.get(*asset_id).unwrap(),
                geometry: BlasGeometries::TriangleGeometries(vec![BlasTriangleGeometry {
                    size: &blas_size,
                    vertex_buffer: &gpu_mesh.vertex_buffer,
                    first_vertex: 0,
                    vertex_stride: gpu_mesh.layout.layout().array_stride,
                    index_buffer: Some(index_buffer),
                    index_buffer_offset: Some(0),
                    transform_buffer: None,
                    transform_buffer_offset: None,
                }]),
            },
        )
        .collect::<Vec<_>>();

    // Build geometry into each BLAS
    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("build_blas_command_encoder"),
    });
    command_encoder.build_acceleration_structures(&build_entries, &[]);
    render_queue.submit([command_encoder.finish()]);
}

fn setup_blas<'a, 'b>(
    asset_id: &'a AssetId<Mesh>,
    gpu_mesh: &'a GpuMesh,
    blas_manager: &'b mut BlasManager,
    render_device: &'b RenderDevice,
) -> (
    &'a AssetId<Mesh>,
    &'a GpuMesh,
    BlasTriangleGeometrySizeDescriptor,
    &'a Buffer,
) {
    let (index_buffer, index_count) = {
        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed { buffer, count, .. } => (buffer, Some(*count)),
            GpuBufferInfo::NonIndexed => unreachable!(),
        }
    };

    let blas_size = BlasTriangleGeometrySizeDescriptor {
        vertex_format: Mesh::ATTRIBUTE_POSITION.format,
        vertex_count: gpu_mesh.vertex_count,
        index_format: Some(IndexFormat::Uint32),
        index_count,
        flags: AccelerationStructureGeometryFlags::OPAQUE,
    };

    let blas = render_device.wgpu_device().create_blas(
        &CreateBlasDescriptor {
            label: Some("blas"),
            flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: AccelerationStructureUpdateMode::Build,
        },
        BlasGeometrySizeDescriptors::Triangles {
            desc: vec![blas_size.clone()],
        },
    );
    blas_manager.blas.insert(*asset_id, blas);

    (asset_id, gpu_mesh, blas_size, index_buffer)
}

fn mesh_compatible(gpu_mesh: &GpuMesh) -> bool {
    let triangle_list = gpu_mesh.primitive_topology == PrimitiveTopology::TriangleList;
    let vertex_layout = gpu_mesh.layout.attribute_ids()
        == &[
            Mesh::ATTRIBUTE_POSITION.id,
            Mesh::ATTRIBUTE_NORMAL.id,
            Mesh::ATTRIBUTE_UV_0.id,
            Mesh::ATTRIBUTE_TANGENT.id,
        ];
    let indexed_32 = matches!(
        gpu_mesh.buffer_info,
        GpuBufferInfo::Indexed {
            index_format: IndexFormat::Uint32,
            ..
        }
    );
    triangle_list && vertex_layout && indexed_32 && gpu_mesh.ray_tracing_support
}
