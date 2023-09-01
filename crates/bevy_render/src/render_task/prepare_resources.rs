use super::RenderTask;
use crate::texture::{CachedTexture, TextureCache};
use bevy_core::FrameCount;
use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Query, Res, ResMut, Resource},
};
use bevy_math::UVec2;
use bevy_utils::{HashMap, HashSet};
use wgpu::{
    Extent3d, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView,
};

pub enum RenderTaskResource {
    Texture {
        format: TextureFormat,
        width: u32,
        height: u32,
        mip_count: u32,
        layer_count: u32,
        dimension: TextureDimension,
    },
    Sampler(Box<SamplerDescriptor<'static>>),
}

impl RenderTaskResource {
    pub fn texture_2d(size: UVec2, format: TextureFormat) -> Self {
        Self::Texture {
            format,
            width: size.x,
            height: size.y,
            mip_count: 1,
            layer_count: 1,
            dimension: TextureDimension::D2,
        }
    }
}

pub enum RenderTaskResourceView {
    SampledTexture {
        name: &'static str,
        mip: u32,
        layer: u32,
        previous_frame: bool,
    },
    StorageTextureWrite {
        name: &'static str,
        mip: u32,
        layer: u32,
        previous_frame: bool,
    },
    StorageTextureReadWrite {
        name: &'static str,
        mip: u32,
        layer: u32,
        previous_frame: bool,
    },
    Sampler(&'static str),
}

impl RenderTaskResourceView {
    pub fn sampled_texture(name: &'static str) -> Self {
        Self::SampledTexture {
            name,
            mip: 0,
            layer: 0,
            previous_frame: false,
        }
    }

    pub fn storage_texture_write(name: &'static str) -> Self {
        Self::StorageTextureWrite {
            name,
            mip: 0,
            layer: 0,
            previous_frame: false,
        }
    }

    pub fn storage_texture_read_write(name: &'static str) -> Self {
        Self::StorageTextureReadWrite {
            name,
            mip: 0,
            layer: 0,
            previous_frame: false,
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

pub fn prepare_resources<R: RenderTask>(
    query: Query<Entity, With<R::RenderTaskSettings>>,
    texture_cache: TextureCache,
    frame_count: Res<FrameCount>,
) {
    let mut texture_descriptors = HashMap::new();
    let mut sampler_descriptors = HashMap::new();

    for (name, resource) in R::resources() {
        match resource {
            RenderTaskResource::Texture {
                format,
                width,
                height,
                mip_count,
                layer_count,
                dimension,
            } => {
                let descriptor = TextureDescriptor {
                    label: Some(name),
                    size: Extent3d {
                        width,
                        height,
                        depth_or_array_layers: layer_count,
                    },
                    mip_level_count: mip_count,
                    sample_count: 1,
                    dimension,
                    format,
                    usage: TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                };
                texture_descriptors.insert(name.to_string(), descriptor);
            }
            RenderTaskResource::Sampler(descriptor) => {
                sampler_descriptors.insert(name, *descriptor);
            }
        }
    }

    let mut double_buffer = HashSet::new();
    for (_, pass) in R::passes() {
        for resource_view in pass.bindings {
            match resource_view {
                RenderTaskResourceView::SampledTexture {
                    name,
                    previous_frame,
                    ..
                } => {
                    if *previous_frame {
                        double_buffer.insert(name);
                    }
                }
                RenderTaskResourceView::StorageTextureWrite {
                    name,
                    previous_frame,
                    ..
                } => {
                    texture_descriptors.get_mut(*name).unwrap().usage |=
                        TextureUsages::STORAGE_BINDING;
                    if *previous_frame {
                        double_buffer.insert(name);
                    }
                }
                RenderTaskResourceView::StorageTextureReadWrite {
                    name,
                    previous_frame,
                    ..
                } => {
                    texture_descriptors.get_mut(*name).unwrap().usage |=
                        TextureUsages::STORAGE_BINDING;
                    if *previous_frame {
                        double_buffer.insert(name);
                    }
                }
                RenderTaskResourceView::Sampler(_) => {}
            }
        }
    }

    for name in double_buffer {
        let descriptor = texture_descriptors.remove(*name).unwrap();
        let descriptor_1 = TextureDescriptor {
            label: Some(&format!("{name}_1")),
            ..descriptor
        };
        let descriptor_2 = TextureDescriptor {
            label: Some(&format!("{name}_2")),
            ..descriptor
        };

        if frame_count.0 % 2 == 0 {
            texture_descriptors.insert(format!("{name}_previous"), descriptor_1);
            texture_descriptors.insert(format!("{name}_current"), descriptor_2);
        } else {
            texture_descriptors.insert(format!("{name}_previous"), descriptor_2);
            texture_descriptors.insert(format!("{name}_current"), descriptor_1);
        }
    }

    for (name, sampler_descriptor) in sampler_descriptors {
        // TODO: Sampler creation needs to be moved to static resources or something
    }
    for entity in &query {
        for (name, texture_descriptor) in texture_descriptors {
            // TODO: Create textures and put in internal registry
        }
    }
}
