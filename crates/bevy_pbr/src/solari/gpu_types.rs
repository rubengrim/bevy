use bevy_asset::AssetId;
use bevy_render::{color::Color, render_resource::ShaderType, texture::Image};

pub struct SolariMaterial {
    pub base_color: Color,
    pub base_color_texture: Option<AssetId<Image>>,
    pub normal_map_texture: Option<AssetId<Image>>,
    pub emissive: Color,
    pub emissive_texture: Option<AssetId<Image>>,
}

#[derive(ShaderType)]
pub struct GpuSolariMaterial {
    pub base_color: [f32; 4],
    pub base_color_texture_id: u32,
    pub normal_map_texture_id: u32,
    pub emissive: [f32; 4],
    pub emissive_texture_id: u32,
}
