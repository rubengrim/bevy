use super::{prepare_pipelines::RenderTaskPipelineIds, RenderTask};
use crate::{
    render_graph::{Node, NodeRunError, RenderGraphContext},
    render_resource::PipelineCache,
    renderer::RenderContext,
};
use bevy_ecs::world::{FromWorld, World};
use bevy_utils::HashMap;
use std::marker::PhantomData;

pub struct RenderTaskNode<R: RenderTask> {
    _marker: PhantomData<R>,
}

impl<R: RenderTask> Node for RenderTaskNode<R> {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_ids = world
            .entity(graph.view_entity()) // TODO: Is using view_entity correct here? Should use ViewNode if so
            .get::<RenderTaskPipelineIds<R>>()
            .unwrap();

        let mut pipelines = HashMap::new();
        for (key, pipeline_id) in &pipeline_ids.ids {
            pipelines.insert(
                *key,
                pipeline_cache.get_compute_pipeline(*pipeline_id).unwrap(),
            );
        }

        R::encode_commands(render_context.command_encoder(), pipelines);

        Ok(())
    }
}

impl<R: RenderTask> FromWorld for RenderTaskNode<R> {
    fn from_world(_world: &mut World) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
