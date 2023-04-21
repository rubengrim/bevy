use bevy_render::{
    render_resource::{encase::private::WriteInto, ShaderSize, StorageBuffer},
    renderer::{RenderDevice, RenderQueue},
};
use bevy_transform::prelude::GlobalTransform;
use bevy_utils::HashMap;
use std::hash::Hash;

pub struct IndexedVec<T, I: Hash + Eq + Clone> {
    pub vec: Vec<T>,
    pub index: HashMap<I, u32>,
}

impl<T, I: Hash + Eq + Clone> IndexedVec<T, I> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn get_index<F: FnOnce(I) -> T>(&mut self, index_key: I, create_value: F) -> u32 {
        *self.index.entry(index_key.clone()).or_insert_with(|| {
            let i = self.vec.len() as u32;
            self.vec.push(create_value(index_key));
            i
        })
    }
}

pub fn new_storage_buffer<T: ShaderSize + WriteInto>(
    vec: Vec<T>,
    label: &'static str,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) -> StorageBuffer<Vec<T>> {
    let mut buffer = StorageBuffer::default();
    buffer.set(vec);
    buffer.set_label(Some(label));
    buffer.write_buffer(render_device, render_queue);
    buffer
}

pub fn tlas_transform(transform: &GlobalTransform) -> [f32; 12] {
    transform.compute_matrix().transpose().to_cols_array()[..12]
        .try_into()
        .unwrap()
}
