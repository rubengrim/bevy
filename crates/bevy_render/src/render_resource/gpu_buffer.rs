use super::StorageBuffer;
use crate::{
    render_resource::BatchedUniformBuffer,
    renderer::{RenderDevice, RenderQueue},
};
use bevy_ecs::{prelude::Component, system::Resource};
use encase::{private::WriteInto, ShaderSize, ShaderType};
use std::{marker::PhantomData, mem};
use wgpu::{BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, ShaderStages};

pub trait GpuBufferable: ShaderType + ShaderSize + WriteInto + Clone {}
impl<T: ShaderType + ShaderSize + WriteInto + Clone> GpuBufferable for T {}

#[derive(Resource)]
pub enum GpuBuffer<T: GpuBufferable> {
    Uniform(BatchedUniformBuffer<T>),
    Storage((StorageBuffer<Vec<T>>, Vec<T>)),
}

impl<T: GpuBufferable> GpuBuffer<T> {
    pub fn new(device: &RenderDevice) -> Self {
        let limits = device.limits();
        if limits.max_storage_buffers_per_shader_stage < 3 {
            GpuBuffer::Uniform(BatchedUniformBuffer::new(&limits))
        } else {
            GpuBuffer::Storage((StorageBuffer::default(), Vec::new()))
        }
    }

    pub fn clear(&mut self) {
        match self {
            GpuBuffer::Uniform(buffer) => buffer.clear(),
            GpuBuffer::Storage((_, buffer)) => buffer.clear(),
        }
    }

    pub fn push(&mut self, value: T) -> GpuBufferIndex<T> {
        match self {
            GpuBuffer::Uniform(buffer) => buffer.push(value),
            GpuBuffer::Storage((_, buffer)) => {
                let instance_index = buffer.len() as u32;
                buffer.push(value);
                GpuBufferIndex {
                    instance_index,
                    dynamic_offset: None,
                    element_type: PhantomData,
                }
            }
        }
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        match self {
            GpuBuffer::Uniform(buffer) => buffer.write_buffer(device, queue),
            GpuBuffer::Storage((buffer, vec)) => {
                buffer.set(mem::take(vec));
                buffer.write_buffer(device, queue);
            }
        }
    }

    pub fn binding_layout(
        binding: u32,
        visibility: ShaderStages,
        device: &RenderDevice,
    ) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility,
            ty: if device.limits().max_storage_buffers_per_shader_stage < 3 {
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(T::min_size()),
                }
            } else {
                BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(T::min_size()),
                }
            },
            count: None,
        }
    }

    pub fn binding(&self) -> Option<BindingResource> {
        match self {
            GpuBuffer::Uniform(buffer) => buffer.binding(),
            GpuBuffer::Storage((buffer, _)) => buffer.binding(),
        }
    }

    pub fn batch_size(device: &RenderDevice) -> Option<u32> {
        let limits = device.limits();
        if limits.max_storage_buffers_per_shader_stage < 3 {
            Some(BatchedUniformBuffer::<T>::batch_size(&limits) as u32)
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct GpuBufferIndex<T: GpuBufferable> {
    pub instance_index: u32,
    pub dynamic_offset: Option<u32>,
    pub element_type: PhantomData<T>,
}
