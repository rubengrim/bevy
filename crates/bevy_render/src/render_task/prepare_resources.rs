use super::RenderTask;
use crate::texture::CachedTexture;
use bevy_ecs::{
    entity::Entity,
    system::{ResMut, Resource},
};
use bevy_math::UVec2;
use bevy_utils::HashMap;
use wgpu::{SamplerDescriptor, TextureDimension, TextureFormat, TextureView};

pub enum RenderTaskResource {
    Texture {
        name: &'static str,
        format: TextureFormat,
        width: u32,
        height: u32,
        mip_count: u32,
        layer_count: u32,
        double_buffered: bool, // TODO: Can infer this from RenderTask::passes()
        dimension: TextureDimension,
    },
    Sampler(Box<SamplerDescriptor<'static>>),
}

impl RenderTaskResource {
    pub fn texture_2d(name: &'static str, size: UVec2, format: TextureFormat) -> Self {
        Self::Texture {
            name,
            format,
            width: size.x,
            height: size.y,
            mip_count: 1,
            layer_count: 1,
            double_buffered: false,
            dimension: TextureDimension::D2,
        }
    }
}

pub enum RenderTaskResourceView {
    SampledTexture {
        name: &'static str,
        mip: u32,
        layer: u32,
    },
    StorageTextureWrite {
        name: &'static str,
        mip: u32,
        layer: u32,
    },
    StorageTextureReadWrite {
        name: &'static str,
        mip: u32,
        layer: u32,
    },
    Sampler(&'static str),
}

impl RenderTaskResourceView {
    pub fn sampled_texture(name: &'static str) -> Self {
        Self::SampledTexture {
            name,
            mip: 0,
            layer: 0,
        }
    }

    pub fn storage_texture_write(name: &'static str) -> Self {
        Self::StorageTextureWrite {
            name,
            mip: 0,
            layer: 0,
        }
    }

    pub fn storage_texture_read_write(name: &'static str) -> Self {
        Self::StorageTextureReadWrite {
            name,
            mip: 0,
            layer: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct RenderTaskResourceRegistry {
    internal: HashMap<(Entity, &'static str), CachedTexture>,
    external: HashMap<(Entity, &'static str), TextureView>,
}

impl RenderTaskResourceRegistry {
    pub fn register_external(&mut self, label: &'static str, entity: Entity, texture: TextureView) {
        let key = (entity, label);
        debug_assert!(!self.external.contains_key(&key));
        self.external.insert(key, texture);
    }

    pub fn get_render_task_resource(
        &self,
        label: &'static str,
        entity: Entity,
    ) -> Option<&CachedTexture> {
        self.internal.get(&(entity, label))
    }

    pub(crate) fn clear(mut this: ResMut<Self>) {
        this.internal.clear();
        this.external.clear();
    }
}

pub fn prepare_resources<R: RenderTask>() {
    // TODO: Loop over resources, map to texture descriptors
    // TODO: Loop over entities, then loop over texture descriptors, create textures, put in internal registry
}
