mod batched_uniform_buffer;

pub use batched_uniform_buffer::*;

use encase::{ShaderSize, ShaderType};
use std::marker::PhantomData;

pub struct BufferIndex<T: ShaderType + ShaderSize> {
    /// Index of an element in the array for use within a shader.
    pub array_index: u32,
    /// Optional dynamic offset to use when binding the buffer.
    pub dynamic_offset: Option<u32>,
    array_type: PhantomData<T>,
}
