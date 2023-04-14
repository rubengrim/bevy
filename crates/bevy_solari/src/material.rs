use bevy_ecs::prelude::Component;
use bevy_math::Vec4;
use bevy_reflect::TypeUuid;
use bevy_render::{prelude::Color, render_resource::ShaderType};

#[derive(Component, TypeUuid, Clone, Default)]
#[uuid = "e624906b-3aa1-437f-ab7b-43a692adf4ff"]
pub struct SolariMaterial {
    pub base_color: Color,
}

#[derive(ShaderType)]
pub struct GpuSolariMaterial {
    base_color: Vec4,
}

impl From<&SolariMaterial> for GpuSolariMaterial {
    fn from(m: &SolariMaterial) -> Self {
        Self {
            base_color: m.base_color.as_linear_rgba_f32().into(),
        }
    }
}
