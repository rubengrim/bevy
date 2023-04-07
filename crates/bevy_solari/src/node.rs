use bevy_ecs::world::{FromWorld, World};
use bevy_render::{
    render_graph::{Node, NodeRunError, RenderGraphContext},
    renderer::RenderContext,
};

pub struct SolariNode {}

impl Node for SolariNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        todo!()
    }
}

impl FromWorld for SolariNode {
    fn from_world(world: &mut World) -> Self {
        Self {}
    }
}
