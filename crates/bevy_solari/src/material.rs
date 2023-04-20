use bevy_asset::Handle;
use bevy_ecs::prelude::Component;
use bevy_math::{Vec3, Vec4};
use bevy_reflect::TypeUuid;
use bevy_render::{prelude::Color, render_resource::ShaderType, texture::Image};

#[derive(Component, TypeUuid, Clone, Default)]
#[uuid = "e624906b-3aa1-437f-ab7b-43a692adf4ff"]
pub struct SolariMaterial {
    pub base_color: Color,
    pub base_color_map: Option<Handle<Image>>,
    pub emission: Option<Color>,
}

#[derive(ShaderType)]
pub struct GpuSolariMaterial {
    pub base_color: Vec4,
    pub base_color_map_index: u32,
    pub emission: Vec3,
}
