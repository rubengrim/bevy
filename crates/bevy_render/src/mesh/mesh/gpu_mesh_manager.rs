use super::Mesh;
use crate::renderer::RenderDevice;
use bevy_asset::Handle;
use bevy_ecs::{
    system::Resource,
    world::{FromWorld, World},
};
use bevy_utils::HashMap;

#[derive(Resource)]
pub struct GpuMeshManager {
    mode: GpuMeshManagerMode,
    metadata: HashMap<Handle<Mesh>, GpuMeshMetadata>,
}

enum GpuMeshManagerMode {
    SingleBuffer,
    MultiBuffer,    
}

impl FromWorld for GpuMeshManager {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        if device.limits().max_storage_buffers_per_shader_stage == 0 {
            Self {
                mode: GpuMeshManagerMode::SingleBuffer,
                metadata: HashMap::new(),
            }
        } else {
            Self {
                mode: GpuMeshManagerMode::MultiBuffer,
                metadata: HashMap::new(),
            }
        }
    }
}

impl GpuMeshManager {
    pub fn insert_or_update(&mut self, asset: &Handle<Mesh>, mesh: &Mesh) {}

    pub fn remove(&mut self, asset: &Handle<Mesh>) {}

    pub fn get(&self, asset: &Handle<Mesh>) {}
}

pub struct GpuMeshMetadata {}

pub fn extract_new_meshes() {}

pub fn prepare_new_meshes() {}
