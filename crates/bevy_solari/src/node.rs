use crate::{misc::ViewBindGroup, pipeline::SolariPipelineId};
use bevy_ecs::{
    query::QueryState,
    world::{FromWorld, World},
};
use bevy_render::{
    camera::ExtractedCamera,
    render_graph::{Node, NodeRunError, RenderGraphContext},
    render_resource::{ComputePassDescriptor, PipelineCache},
    renderer::RenderContext,
    view::ViewUniformOffset,
};

pub struct SolariNode {
    view_query: QueryState<(
        &'static SolariPipelineId,
        &'static ViewBindGroup,
        &'static ViewUniformOffset,
        &'static ExtractedCamera,
    )>,
}

impl Node for SolariNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (
            Ok((pipeline_id, view_bind_group, view_uniform_offset, camera)),
            Some(pipeline_cache),
        ) = (
            self.view_query.get_manual(world, graph.view_entity()),
            world.get_resource::<PipelineCache>(),
        ) else {
            return Ok(());
        };
        let (Some(pipeline), Some(viewport)) = (pipeline_cache.get_compute_pipeline(pipeline_id.0), camera.physical_viewport_size) else {
            return Ok(());
        };

        {
            let mut solari_pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("solari_pass"),
                    });

            solari_pass.set_pipeline(pipeline);
            solari_pass.set_bind_group(0, &view_bind_group.0, &[view_uniform_offset.offset]);
            solari_pass.dispatch_workgroups((viewport.x + 7) / 8, (viewport.y + 7) / 8, 1);
        }

        Ok(())
    }

    fn update(&mut self, world: &mut World) {
        self.view_query.update_archetypes(world);
    }
}

impl FromWorld for SolariNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            view_query: QueryState::new(world),
        }
    }
}
