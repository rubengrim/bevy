use super::BufferIndex;
use crate::{
    render_resource::{Buffer, DynamicUniformBuffer},
    renderer::{RenderDevice, RenderQueue},
};
use bevy_ecs::system::Resource;
use encase::{private::WriteInto, ShaderSize, ShaderType};
use std::{mem, num::NonZeroU64};
use wgpu::{BindingResource, Limits};

/// Stores data to be sent to the GPU in a [`DynamicUniformBuffer`],
/// where each entry in the buffer is an array of values batched together.
///
/// This is essentially a `DynamicUniformBuffer<[T; N]>`.
///
/// Rust usage: TODO
///
/// WGSL usage: TODO
#[derive(Resource)]
pub struct BatchedUniformBuffer<T: ShaderType + ShaderSize> {
    buffer: DynamicUniformBuffer<Vec<T>>,
    current_batch: Vec<T>,
    current_dynamic_offset: u32,
    batch_size: u32,
}

impl<T: ShaderType + ShaderSize + WriteInto> BatchedUniformBuffer<T> {
    pub fn min_element_size() -> NonZeroU64 {
        todo!()
    }

    pub fn new(limits: &Limits) -> Self {
        let batch_size = todo!("limits.max_uniform_buffer_binding_size");
        Self {
            buffer: DynamicUniformBuffer::default(),
            current_batch: Vec::with_capacity(batch_size as usize / 4),
            current_dynamic_offset: 0,
            batch_size,
        }
    }

    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.buffer()
    }

    pub fn binding(&self) -> Option<BindingResource> {
        self.buffer.binding()
    }

    pub fn push(&mut self, value: T) -> BufferIndex<T> {
        let len = self.current_batch.len() as u32;

        if len == self.batch_size {
            self.push_batch();
        }

        self.current_batch.push(value);

        BufferIndex {
            array_index: len,
            dynamic_offset: Some(self.current_dynamic_offset),
            array_type: Default::default(),
        }
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.push_batch();

        self.buffer.write_buffer(device, queue);
    }

    pub fn clear(&mut self) {
        self.current_batch.clear();
        self.buffer.clear();
    }

    fn push_batch(&mut self) {
        let current_batch = mem::take(&mut self.current_batch);

        self.buffer.push(current_batch);

        self.current_dynamic_offset += 1;
    }
}
