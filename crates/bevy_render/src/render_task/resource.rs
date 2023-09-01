use crate::render_resource::TextureView;
use bevy_math::UVec2;
use wgpu::{SamplerDescriptor, TextureDimension, TextureFormat};

pub enum RenderTaskResource {
    TextureRead(RenderTaskTexture),
    TextureWrite(RenderTaskTexture),
    TextureReadWrite(RenderTaskTexture),
    ExternalTextureRead(TextureView),
    ExternalTextureWrite(TextureView),
    ExternalTextureReadWrite(TextureView),
    Sampler(SamplerDescriptor<'static>),
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
