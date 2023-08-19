use bevy_core::FrameCount;
use bevy_ecs::prelude::{Bundle, Component};
use bevy_math::Vec3;
use bevy_render::{
    prelude::Color,
    render_resource::{ShaderType, UniformBuffer},
    renderer::{RenderDevice, RenderQueue},
};
use bevy_transform::prelude::{GlobalTransform, Transform};

#[derive(Bundle, Default)]
pub struct SolariSunBundle {
    pub sun: SolariSun,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Component, Clone)]
pub struct SolariSun {
    pub illuminance: f32,
    pub color: Color,
}

impl Default for SolariSun {
    fn default() -> Self {
        Self {
            illuminance: 100000.0,
            color: Color::rgb_linear(1.0, 1.0, 1.0),
        }
    }
}

#[derive(ShaderType)]
pub struct SolariUniforms {
    frame_count: u32,
    sun_direction: Vec3,
    sun_color: Vec3,
}

impl SolariUniforms {
    pub fn new(
        frame_count: &FrameCount,
        sun: (&SolariSun, &GlobalTransform),
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
    ) -> UniformBuffer<SolariUniforms> {
        let sun_color = sun.0.color.as_linear_rgba_f32();
        let uniforms = Self {
            frame_count: frame_count.0,
            sun_direction: sun.1.back(),
            sun_color: Vec3::new(sun_color[0], sun_color[1], sun_color[2]) * sun.0.illuminance,
        };

        let mut buffer = UniformBuffer::from(uniforms);
        buffer.set_label(Some("solari_uniforms"));
        buffer.write_buffer(render_device, render_queue);
        buffer
    }
}
