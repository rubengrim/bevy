use bevy_ecs::{
    query::QueryState,
    world::{FromWorld, World},
};
use bevy_render::{
    render_graph::{Node, NodeRunError, RenderGraphContext},
    renderer::RenderContext,
};

pub struct SolariWorldCacheNode(QueryState<()>);

impl Node for SolariWorldCacheNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        Ok(())
    }

    fn update(&mut self, world: &mut World) {
        self.0.update_archetypes(world);
    }
}

impl FromWorld for SolariWorldCacheNode {
    fn from_world(world: &mut World) -> Self {
        Self(QueryState::new(world))
    }
}
