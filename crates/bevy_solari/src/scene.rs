use crate::{
    blas::BlasStorage,
    material::{GpuSolariMaterial, SolariMaterial},
};
use bevy_asset::{Assets, Handle};
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use bevy_render::{
    mesh::GpuBufferInfo,
    prelude::Mesh,
    render_asset::RenderAssets,
    render_resource::{
        raytrace::{
            AccelerationStructureFlags, AccelerationStructureUpdateMode, CreateTlasDescriptor,
            DeviceRayTracing,
        },
        BindGroup, ShaderType,
    },
    renderer::{RenderDevice, RenderQueue},
    texture::Image,
};
use bevy_utils::HashMap;
use std::hash::Hash;

#[derive(Resource, Default)]
pub struct SceneBindGroup(pub Option<BindGroup>);

pub fn queue_scene_bind_group(
    objects: Query<(&Handle<Mesh>, &Handle<SolariMaterial>)>,
    scene_bind_group: ResMut<SceneBindGroup>,
    mesh_assets: Res<RenderAssets<Mesh>>,
    material_assets: Res<Assets<SolariMaterial>>,
    image_assets: Res<RenderAssets<Image>>,
    blas_storage: Res<BlasStorage>,
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

    let mut get_index_buffer_i = |mesh_handle| {
        index_buffers.get_index_or_push(mesh_handle, |mesh_handle| {
            let gpu_mesh = mesh_assets.get(mesh_handle).unwrap(); // TODO: Handle unwrap
            vertex_buffers.push(&gpu_mesh.vertex_buffer);
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed { buffer, .. } => buffer,
                _ => unreachable!(), // TODO: Handle non-indexed meshes
            }
        })
    };

    let mut get_texture_map_i = |maybe_texture_map_handle: &Option<Handle<Image>>| {
        if let Some(texture_map_handle) = maybe_texture_map_handle {
            texture_maps.get_index_or_push(texture_map_handle, |_| {
                // TODO: Handle unwrap
                &image_assets.get(texture_map_handle).unwrap().texture_view
            })
        } else {
            u32::MAX
        }
    };

    let mut get_material_i = |material_handle| {
        materials.get_index_or_push(material_handle, |material_handle| {
            let material = material_assets.get(material_handle).unwrap(); // TODO: Handle unwrap
            GpuSolariMaterial {
                base_color: material.base_color.as_linear_rgba_f32().into(),
                base_color_map_index: get_texture_map_i(&material.base_color_map),
            }
        })
    };

    // Create GPU scene resources
    // TODO: Reuse non-TLAS resources each frame
    let tlas = render_device
        .wgpu_device()
        .create_tlas(&CreateTlasDescriptor {
            label: Some("tlas"),
            flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: AccelerationStructureUpdateMode::Build,
            max_instances: objects.len() as u32,
        });

    for (mesh_handle, material_handle) in objects {
        let object_i = mesh_materials.get_index_or_push(
            (mesh_handle, material_handle),
            |(mesh_handle, material_handle)| MeshMaterial {
                index_buffer_i: get_index_buffer_i(mesh_handle),
                material_i: get_material_i(material_handle),
            },
        );
    }
}

struct IndexedVec<T, I: Hash + Eq + Copy> {
    vec: Vec<T>,
    index: HashMap<I, u32>,
}

impl<T, I: Hash + Eq + Copy> IndexedVec<T, I> {
    fn new() -> Self {
        Self {
            vec: Vec::new(),
            index: HashMap::new(),
        }
    }

    fn get_index_or_push<F: FnOnce(I) -> T>(&mut self, index_key: I, create_value: F) -> u32 {
        *self.index.entry(index_key).or_insert_with(|| {
            let i = self.vec.len() as u32;
            self.vec.push(create_value(index_key));
            i
        })
    }
}

#[derive(ShaderType)]
pub struct MeshMaterial {
    index_buffer_i: u32,
    material_i: u32,
}
