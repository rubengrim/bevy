use crate::material::{GpuSolariMaterial, SolariMaterial};
use bevy_ecs::{
    prelude::Component,
    system::{Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::{
    render_resource::{BindingResource, StorageBuffer},
    renderer::{RenderDevice, RenderQueue},
};
use std::mem;

#[derive(Resource)]
pub struct MaterialBuffer {
    cpu_buffer: Vec<GpuSolariMaterial>,
    gpu_buffer: StorageBuffer<Vec<GpuSolariMaterial>>,
}

impl FromWorld for MaterialBuffer {
    fn from_world(_: &mut World) -> Self {
        let mut gpu_buffer = StorageBuffer::<Vec<GpuSolariMaterial>>::default();
        gpu_buffer.set_label(Some("material_buffer"));
        Self {
            cpu_buffer: Vec::new(),
            gpu_buffer,
        }
    }
}

#[derive(Component)]
pub struct MaterialIndex(pub u32);

impl MaterialBuffer {
    pub fn push(&mut self, material: &SolariMaterial) -> MaterialIndex {
        let i = MaterialIndex(self.cpu_buffer.len() as u32);
        self.cpu_buffer.push(material.into());
        i
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
}

pub fn prepare_material_buffer(
    mut material_buffer: ResMut<MaterialBuffer>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    material_buffer.write_buffer(&render_device, &render_queue);
}
