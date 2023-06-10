use bevy_asset::Handle;
use bevy_ecs::prelude::{Bundle, Component};
use bevy_math::{Vec3, Vec4};
use bevy_reflect::TypeUuid;
use bevy_render::{
    prelude::{Color, Mesh},
    render_resource::ShaderType,
    texture::Image,
};
use bevy_transform::prelude::{GlobalTransform, Transform};

#[derive(Component, TypeUuid, Clone, Default)]
#[uuid = "e624906b-3aa1-437f-ab7b-43a692adf4ff"]
pub struct SolariMaterial {
    pub base_color: Color,
    pub base_color_map: Option<Handle<Image>>,
    pub normal_map: Option<Handle<Image>>,
    pub emission: Option<Color>,
}

#[derive(Bundle, Clone, Default)]
pub struct SolariMaterialMeshBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<SolariMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(ShaderType)]
pub struct GpuSolariMaterial {
    pub base_color: Vec4,
    pub base_color_map_index: u32,
    pub normal_map_index: u32,
    pub emission: Vec3,
}
