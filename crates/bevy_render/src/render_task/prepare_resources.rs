use super::RenderTask;
use crate::{
    camera::ExtractedCamera,
    render_resource::{Texture, TextureDescriptorOwned},
    renderer::RenderDevice,
};
use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Query, Res, ResMut, Resource},
};
use bevy_math::UVec2;
use bevy_utils::{HashMap, HashSet};
use wgpu::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
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
    internal: HashMap<(Entity, String), Texture>,
    external: HashMap<(Entity, &'static str), TextureView>,
}

impl RenderTaskResourceRegistry {
    pub fn register_external(&mut self, label: &'static str, entity: Entity, texture: TextureView) {
        let key = (entity, label);
        debug_assert!(!self.external.contains_key(&key));
        self.external.insert(key, texture);
    }

    pub fn get_render_task_resource<R: RenderTask>(
        &self,
        name: &str,
        entity: Entity,
    ) -> Option<&Texture> {
        let name = format!("{}_{name}", R::name());
        self.internal.get(&(entity, name))
    }

    pub(crate) fn cleanup(mut this: ResMut<Self>) {
        this.external.clear();

        // TODO: Clear this.internal values that are no longer needed
    }
}

pub fn prepare_resources<R: RenderTask>(
    query: Query<(Entity, &ExtractedCamera), With<R::RenderTaskSettings>>,
    mut resource_registry: ResMut<RenderTaskResourceRegistry>,
    render_device: Res<RenderDevice>,
) {
    let task_name = R::name();
    for (entity, camera) in &query {
        let Some(physical_viewport_size) = camera.physical_viewport_size else { continue };
        let mut texture_descriptors = HashMap::new();

        // Setup initial resource descriptors
        for (name, resource) in R::resources(physical_viewport_size) {
            match resource {
                RenderTaskResource::Texture {
                    format,
                    width,
                    height,
                    mip_count,
                    layer_count,
                    dimension,
                } => {
                    let descriptor = TextureDescriptorOwned {
                        label: format!("{task_name}_{name}"),
                        size: Extent3d {
                            width,
                            height,
                            depth_or_array_layers: layer_count,
                        },
                        mip_level_count: mip_count,
                        sample_count: 1,
                        dimension,
                        format,
                        usage: TextureUsages::empty(),
                        view_formats: &[],
                    };
                    texture_descriptors.insert(name.to_string(), descriptor);
                }
            }
        }

        // Fill out resource usages and find double buffered resources based on passes
        let mut double_buffer = HashSet::new();
        for (_, pass) in R::passes() {
            for resource_view in pass.bindings {
                match resource_view {
                    RenderTaskResourceView::SampledTexture {
                        name,
                        previous_frame,
                        ..
                    } => {
                        texture_descriptors.get_mut(*name).unwrap().usage |=
                            TextureUsages::TEXTURE_BINDING;
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

        // Split double buffered resources into two descriptors
        for name in double_buffer {
            let descriptor = texture_descriptors.remove(*name).unwrap();

            let name_1 = format!("{task_name}_{name}_1");
            let descriptor_1 = TextureDescriptorOwned {
                label: name_1.clone(),
                ..descriptor
            };
            let name_2 = format!("{task_name}_{name}_2");
            let descriptor_2 = TextureDescriptorOwned {
                label: name_2.clone(),
                ..descriptor
            };

            texture_descriptors.insert(name_1, descriptor_1);
            texture_descriptors.insert(name_2, descriptor_2);
        }

        // Create resources and put in internal registry
        for (name, descriptor) in texture_descriptors {
            let descriptor = TextureDescriptor {
                label: Some(&descriptor.label),
                size: descriptor.size,
                mip_level_count: descriptor.mip_level_count,
                sample_count: descriptor.sample_count,
                dimension: descriptor.dimension,
                format: descriptor.format,
                usage: descriptor.usage,
                view_formats: descriptor.view_formats,
            };

            resource_registry
                .internal
                .entry((entity, format!("{task_name}_{name}")))
                .or_insert_with(|| render_device.create_texture(&descriptor));
        }
    }
}
