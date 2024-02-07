use crate::MeshUniform;
use bevy_ecs::{
    system::Resource,
    world::{FromWorld, World},
};
use bevy_render::{render_resource::*, renderer::RenderDevice};

#[derive(Resource)]
pub struct SceneBindings {
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: Option<BindGroup>,
}

impl FromWorld for SceneBindings {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        Self {
            bind_group_layout: render_device.create_bind_group_layout(
                "solari_scene_bind_group_layout",
                &BindGroupLayoutEntries::sequential(
                    ShaderStages::COMPUTE,
                    (
                        BindingType::AccelerationStructure,
                        // TODO: AS->mesh/material mapping
                        GpuArrayBuffer::<MeshUniform>::binding_layout(render_device),
                        // TODO: Materials
                        // TODO: Lights
                    ),
                ),
            ),
            bind_group: None,
        }
    }
}
