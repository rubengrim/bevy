use bevy_core::FrameCount;
use bevy_ecs::prelude::Component;
use bevy_math::Vec3;
use bevy_render::{
    extract_component::ExtractComponent,
    prelude::Color,
    render_resource::{ShaderType, UniformBuffer},
    renderer::{RenderDevice, RenderQueue},
};

#[derive(Component, ExtractComponent, Clone)]
pub struct SolariSun {
    pub direction: Vec3,
    pub illuminance: f32,
    pub color: Color,
}

impl Default for SolariSun {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.24868992, 0.94525665, 0.21128981),
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
        sun: &SolariSun,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
    ) -> UniformBuffer<SolariUniforms> {
        let sun_color = sun.color.as_linear_rgba_f32();
        let uniforms = Self {
            frame_count: frame_count.0,
            sun_direction: sun.direction,
            sun_color: Vec3::new(sun_color[0], sun_color[1], sun_color[2]) * sun.illuminance,
        };

        let mut buffer = UniformBuffer::from(uniforms);
        buffer.set_label(Some("solari_uniforms"));
        buffer.write_buffer(render_device, render_queue);
        buffer
    }
}
