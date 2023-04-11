use crate::misc::create_view_bind_group_layout;
use bevy_ecs::{
    system::Resource,
    world::{FromWorld, World},
};
use bevy_render::{render_resource::BindGroupLayout, renderer::RenderDevice};

#[derive(Resource)]
pub struct SolariPipeline {
    pub view_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        Self {
            view_bind_group_layout: create_view_bind_group_layout(render_device),
        }
    }
}
