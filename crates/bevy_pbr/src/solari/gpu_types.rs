use bevy_asset::AssetId;
use bevy_render::{color::Color, texture::Image};

pub struct SolariMaterial {
    pub base_color: Color,
    pub base_color_texture: Option<AssetId<Image>>,
    pub normal_map_texture: Option<AssetId<Image>>,
    pub emissive: Color,
    pub emissive_texture: Option<AssetId<Image>>,
}
