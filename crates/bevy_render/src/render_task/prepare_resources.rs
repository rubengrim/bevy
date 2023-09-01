use super::RenderTask;
use crate::texture::CachedTexture;
use bevy_ecs::{entity::Entity, system::Resource};
use bevy_math::UVec2;
use bevy_utils::HashMap;
use wgpu::{SamplerDescriptor, TextureDimension, TextureFormat, TextureView};

pub enum RenderTaskResource {
    TextureRead(RenderTaskTexture),
    TextureWrite(RenderTaskTexture),
    TextureReadWrite(RenderTaskTexture),
    ExternalTextureRead(&'static str),
    ExternalTextureWrite(&'static str),
    ExternalTextureReadWrite(&'static str),
    Sampler(Box<SamplerDescriptor<'static>>),
}

pub struct RenderTaskTexture {
    pub label: &'static str,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub mip: u32,
    pub layer: u32,
    pub from_previous_frame: bool,
    pub dimension: TextureDimension,
}

impl RenderTaskTexture {
    pub fn new(label: &'static str, format: TextureFormat, size: UVec2) -> Self {
        Self {
            label,
            format,
            width: size.x,
            height: size.y,
            mip: 0,
            layer: 0,
            from_previous_frame: false,
            dimension: TextureDimension::D2,
        }
    }
}

#[derive(Resource, Default)]
pub struct RenderTaskResourceRegistry {
    internal: HashMap<(Entity, &'static str), TextureView>,
    external: HashMap<(Entity, &'static str), CachedTexture>,
}

impl RenderTaskResourceRegistry {
    pub fn register_external(&mut self, label: &'static str, entity: Entity, texture: TextureView) {
        let key = (entity, label);
        debug_assert!(!self.internal.contains_key(&key));
        self.internal.insert(key, texture);
    }

    pub fn get_render_task_resource(
        &self,
        label: &'static str,
        entity: Entity,
    ) -> Option<&CachedTexture> {
        self.external.get(&(entity, label))
    }

    fn clear(&mut self) {
        self.internal.clear();
        self.external.clear();
    }
}

pub fn prepare_resources<R: RenderTask>() {
    // TODO: Loop over all R::pipelines(), build up wgpu::TextureDescriptors and then create textures/views
}
