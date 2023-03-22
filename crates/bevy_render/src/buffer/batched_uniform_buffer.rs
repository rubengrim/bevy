use super::BufferIndex;
use crate::{
    render_resource::{Buffer, DynamicUniformBuffer},
    renderer::{RenderDevice, RenderQueue},
};
use bevy_ecs::system::Resource;
use encase::{private::WriteInto, ArrayLength, ShaderSize, ShaderType};
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
    buffer: DynamicUniformBuffer<Batch<T>>,
    current_batch: Vec<T>,
    current_dynamic_offset: u32,
    max_uniform_buffer_binding_size: u32,
}

impl<T: ShaderType + ShaderSize + WriteInto + Clone> BatchedUniformBuffer<T> {
    pub fn min_element_size() -> NonZeroU64 {
        Batch::<T>::min_size()
    }

    pub fn new(limits: &Limits) -> Self {
        Self {
            buffer: DynamicUniformBuffer::default(),
            current_batch: Vec::new(),
            current_dynamic_offset: 0,
            max_uniform_buffer_binding_size: limits.max_uniform_buffer_binding_size,
        }
    }

    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.buffer()
    }

    pub fn binding(&self) -> Option<BindingResource> {
        self.buffer.binding()
    }

    pub fn push(&mut self, value: T) -> BufferIndex<T> {
        if self.batch_full() {
            self.push_batch();
        }

        self.current_batch.push(value);

        BufferIndex {
            array_index: self.current_batch.len() as u32 - 1,
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

    fn batch_full(&self) -> bool {
        todo!()
    }

    fn push_batch(&mut self) {
        let current_batch = mem::take(&mut self.current_batch);

        self.buffer.push(Batch {
            len: ArrayLength,
            values: current_batch,
        });

        self.current_dynamic_offset += 1;
    }
}

/// An entry in [`BatchedUniformBuffer`].
///
/// Each batch can be up to `wgpu::Limits::max_uniform_buffer_binding_size` (default 64kb).
#[derive(ShaderType)]
struct Batch<T: ShaderType + ShaderSize> {
    len: ArrayLength,
    values: Vec<T>,
}
