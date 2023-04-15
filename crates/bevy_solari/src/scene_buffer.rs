use crate::{
    array_buffer::ArrayBuffer,
    binding_array::{BufferBindingArray, TextureBindingArray},
    material::{GpuSolariMaterial, SolariMaterial},
};
use bevy_asset::{Assets, Handle};
use bevy_ecs::prelude::Component;
use bevy_render::{prelude::Mesh, render_asset::RenderAssets, render_resource::ShaderType};

pub struct SceneBuffers<'a> {
    mesh_material_buffer: ArrayBuffer<MeshMaterial, (Handle<Mesh>, Handle<SolariMaterial>)>,
    vertex_buffers: BufferBindingArray<'a>,
    index_buffers: BufferBindingArray<'a>,
    material_buffer: ArrayBuffer<GpuSolariMaterial, Handle<SolariMaterial>>,
    texture_maps: TextureBindingArray<'a>,
}

impl SceneBuffers<'_> {
    fn new() -> Self {
        Self {
            mesh_material_buffer: ArrayBuffer::new("solari_mesh_material_buffer"),
            vertex_buffers: BufferBindingArray::default(),
            index_buffers: BufferBindingArray::default(),
            material_buffer: ArrayBuffer::new("solari_material_buffer"),
            texture_maps: TextureBindingArray::default(),
        }
    }

    pub fn push_mesh_material(
        &mut self,
        mesh: &Handle<Mesh>,
        material: &Handle<SolariMaterial>,
        meshes: &RenderAssets<Mesh>,
        materials: &Assets<SolariMaterial>,
    ) -> Option<MeshMaterialIndex> {
        match (meshes.get(mesh), materials.get(material)) {
            (Some(gpu_mesh), Some(material)) => todo!(),
            _ => None,
        }
    }
}

#[derive(Component)]
pub struct MeshMaterialIndex(pub u32);

#[derive(ShaderType)]
struct MeshMaterial {
    index_i: u32,
    material_i: u32,
}
