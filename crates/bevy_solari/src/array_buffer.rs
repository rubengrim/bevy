use bevy_render::{
    render_resource::{encase::private::WriteInto, BindingResource, ShaderSize, StorageBuffer},
    renderer::{RenderDevice, RenderQueue},
};
use bevy_utils::HashMap;
use std::{hash::Hash, mem};

pub struct ArrayBuffer<T: ShaderSize + WriteInto, I: Hash + Eq> {
    cpu_buffer: Vec<T>,
    gpu_buffer: StorageBuffer<Vec<T>>,
    lookup_table: HashMap<I, u32>,
}

impl<T: ShaderSize + WriteInto, I: Hash + Eq> ArrayBuffer<T, I> {
    pub fn new(label: &'static str) -> Self {
        let mut gpu_buffer = StorageBuffer::<Vec<T>>::default();
        gpu_buffer.set_label(Some(label));

        Self {
            cpu_buffer: Vec::new(),
            gpu_buffer,
            lookup_table: HashMap::new(),
        }
    }

    pub fn push(&mut self, value: T, index: I) -> u32 {
        *self.lookup_table.entry(index).or_insert_with(|| {
            let i = self.cpu_buffer.len() as u32;
            self.cpu_buffer.push(value);
            i
        })
    }

    pub fn write_to_gpu(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        let mut new_cpu_buffer = Vec::with_capacity(self.cpu_buffer.len());
        mem::swap(&mut self.cpu_buffer, &mut new_cpu_buffer);
        self.gpu_buffer.set(new_cpu_buffer);
        self.gpu_buffer.write_buffer(render_device, render_queue);

        self.lookup_table.clear();
    }

    pub fn binding(&self) -> BindingResource<'_> {
        self.gpu_buffer.binding().unwrap()
    }
}
