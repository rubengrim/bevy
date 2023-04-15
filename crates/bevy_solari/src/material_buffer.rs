use crate::material::{GpuSolariMaterial, SolariMaterial};
use bevy_ecs::{
    prelude::Component,
    system::{Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::{
    render_asset::RenderAssets,
    render_resource::{BindingResource, StorageBuffer, TextureView},
    renderer::{RenderDevice, RenderQueue},
    texture::{FallbackImage, Image},
};
use std::mem;

#[derive(Resource)]
pub struct MaterialBuffer {
    cpu_buffer: Vec<GpuSolariMaterial>,
    gpu_buffer: StorageBuffer<Vec<GpuSolariMaterial>>,
    texture_maps: Vec<TextureView>,
    texture_maps_empty: [TextureView; 1],
}

impl FromWorld for MaterialBuffer {
    fn from_world(world: &mut World) -> Self {
        let mut gpu_buffer = StorageBuffer::<Vec<GpuSolariMaterial>>::default();
        gpu_buffer.set_label(Some("material_buffer"));

        let texture_maps_empty = [world.resource::<FallbackImage>().texture_view.clone()];

        Self {
            cpu_buffer: Vec::new(),
            gpu_buffer,
            texture_maps: Vec::new(),
            texture_maps_empty,
        }
    }
}

#[derive(Component)]
pub struct MaterialIndex(pub u32);

impl MaterialBuffer {
    pub fn push(
        &mut self,
        material: &SolariMaterial,
        images: &RenderAssets<Image>,
    ) -> MaterialIndex {
        let i = MaterialIndex(self.cpu_buffer.len() as u32);
        self.cpu_buffer
            .push(material.to_gpu(images, &mut self.texture_maps));
        i
    }

    pub fn clear_texture_maps(&mut self) {
        self.texture_maps.clear();
    }

    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        let mut new_cpu_buffer = Vec::with_capacity(self.cpu_buffer.len());
        mem::swap(&mut self.cpu_buffer, &mut new_cpu_buffer);
        self.gpu_buffer.set(new_cpu_buffer);
        self.gpu_buffer.write_buffer(render_device, render_queue);
    }

    pub fn binding(&self) -> BindingResource<'_> {
        self.gpu_buffer.binding().unwrap()
    }

    pub fn texture_maps(&self) -> &[TextureView] {
        if self.texture_maps.is_empty() {
            &self.texture_maps_empty
        } else {
            &self.texture_maps
        }
    }
}

pub fn prepare_material_buffer(
    mut material_buffer: ResMut<MaterialBuffer>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    material_buffer.write_buffer(&render_device, &render_queue);
}
