use super::{persistent_buffer::PersistentGpuBufferable, Meshlet, MeshletBoundingSphere};
use std::{borrow::Cow, sync::Arc};

impl PersistentGpuBufferable for Arc<[u8]> {
    type ExtraData = ();

    fn size_in_bytes(&self) -> u64 {
        self.len() as u64
    }

    fn as_bytes_le(&self, _: Self::ExtraData) -> Cow<'_, [u8]> {
        Cow::Borrowed(self)
    }
}

impl PersistentGpuBufferable for Arc<[u32]> {
    type ExtraData = u64;

    fn size_in_bytes(&self) -> u64 {
        self.len() as u64 * 4
    }

    fn as_bytes_le(&self, offset: Self::ExtraData) -> Cow<'_, [u8]> {
        let offset = offset as u32 / 48;

        self.iter()
            .flat_map(|index| (*index + offset).to_le_bytes())
            .collect()
    }
}

impl PersistentGpuBufferable for Arc<[Meshlet]> {
    type ExtraData = (u64, u64);

    fn size_in_bytes(&self) -> u64 {
        self.len() as u64 * 12
    }

    fn as_bytes_le(&self, (vertex_offset, index_offset): Self::ExtraData) -> Cow<'_, [u8]> {
        let vertex_offset = vertex_offset as u32 / 4;
        let index_offset = index_offset as u32;

        self.iter()
            .flat_map(|meshlet| {
                bytemuck::cast::<_, [u8; 12]>(Meshlet {
                    start_vertex_id: meshlet.start_vertex_id + vertex_offset,
                    start_index_id: meshlet.start_index_id + index_offset,
                    vertex_count: meshlet.vertex_count,
                })
            })
            .collect()
    }
}

impl PersistentGpuBufferable for Arc<[MeshletBoundingSphere]> {
    type ExtraData = ();

    fn size_in_bytes(&self) -> u64 {
        self.len() as u64 * 16
    }

    fn as_bytes_le(&self, _: Self::ExtraData) -> Cow<'_, [u8]> {
        Cow::Borrowed(bytemuck::cast_slice(self))
    }
}
