use bevy_asset::Handle;
use bevy_ecs::prelude::Component;
use bevy_math::Vec4;
use bevy_reflect::TypeUuid;
use bevy_render::{
    prelude::Color,
    render_asset::RenderAssets,
    render_resource::{ShaderType, TextureView},
    texture::Image,
};

#[derive(Component, TypeUuid, Clone, Default)]
#[uuid = "e624906b-3aa1-437f-ab7b-43a692adf4ff"]
pub struct SolariMaterial {
    pub base_color: Color,
    pub base_color_map: Option<Handle<Image>>,
}

#[derive(ShaderType)]
pub struct GpuSolariMaterial {
    base_color: Vec4,
    base_color_map_index: u32,
}

impl SolariMaterial {
    pub fn to_gpu(
        &self,
        images: &RenderAssets<Image>,
        texture_maps: &mut Vec<TextureView>,
    ) -> GpuSolariMaterial {
        GpuSolariMaterial {
            base_color: self.base_color.as_linear_rgba_f32().into(),
            base_color_map_index: texture_map_index(&self.base_color_map, images, texture_maps),
        }
    }
}

// TODO: Don't create duplicate entries for the same handle
// Pass in a &mut HashMap<Handle<Image>, u32>
fn texture_map_index(
    texture_map: &Option<Handle<Image>>,
    images: &RenderAssets<Image>,
    texture_maps: &mut Vec<TextureView>,
) -> u32 {
    match texture_map
        .as_ref()
        .and_then(|texture_map| images.get(&texture_map))
    {
        Some(gpu_texture_map) => {
            let i = texture_maps.len() as u32;
            assert_ne!(i, u32::MAX, "max textures reached");
            texture_maps.push(gpu_texture_map.texture_view.clone());
            i
        }
        None => u32::MAX,
    }
}
