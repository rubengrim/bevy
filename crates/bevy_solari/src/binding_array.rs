use bevy_render::render_resource::{Buffer, BufferId, TextureView, TextureViewId};
use bevy_utils::HashMap;

#[derive(Default)]
pub struct BufferBindingArray<'a> {
    array: Vec<&'a Buffer>,
    lookup_table: HashMap<BufferId, u32>,
}

impl BufferBindingArray<'_> {
    pub fn push(&mut self, value: &Buffer) -> u32 {
        *self.lookup_table.entry(value.id()).or_insert_with(|| {
            let i = self.array.len() as u32;
            self.array.push(value);
            i
        })
    }

    pub fn clear(&mut self) {
        self.array.clear();
        self.lookup_table.clear();
    }
}

#[derive(Default)]
pub struct TextureBindingArray<'a> {
    array: Vec<&'a TextureView>,
    lookup_table: HashMap<TextureViewId, u32>,
}

impl TextureBindingArray<'_> {
    pub fn push(&mut self, value: &TextureView) -> u32 {
        *self.lookup_table.entry(value.id()).or_insert_with(|| {
            let i = self.array.len() as u32;
            self.array.push(value);
            i
        })
    }

    pub fn clear(&mut self) {
        self.array.clear();
        self.lookup_table.clear();
    }
}
