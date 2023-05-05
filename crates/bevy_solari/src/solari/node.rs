use super::{
    filter_screen_probes::SolariFilterScreenProbesPipelineId, gm_buffer::SolariGmBufferPipelineId,
    resources::SolariBindGroup, update_screen_probes::SolariUpdateScreenProbesPipelineId,
};
use crate::scene::bind_group::SolariSceneBindGroup;
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

pub struct SolariNode(
    QueryState<(
        &'static SolariBindGroup,
        &'static SolariGmBufferPipelineId,
        &'static SolariUpdateScreenProbesPipelineId,
        &'static SolariFilterScreenProbesPipelineId,
        &'static ViewUniformOffset,
        &'static ExtractedCamera,
    )>,
);

impl Node for SolariNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (
            Ok((bind_group, gm_buffer_pipeline_id, update_screen_probes_pipeline_id, filter_screen_probes_pipeline_id, view_uniform_offset, camera)),
            Some(pipeline_cache),
            Some(SolariSceneBindGroup(Some(scene_bind_group))),
        ) = (
            self.0.get_manual(world, graph.view_entity()),
            world.get_resource::<PipelineCache>(),
            world.get_resource::<SolariSceneBindGroup>(),
        ) else {
            return Ok(());
        };
        let (
            Some(gm_buffer_pipeline),
            Some(update_screen_probes_pipeline),
            Some(filter_screen_probes_pipeline),
            Some(viewport),
        ) = (
            pipeline_cache.get_compute_pipeline(gm_buffer_pipeline_id.0),
            pipeline_cache.get_compute_pipeline(update_screen_probes_pipeline_id.0),
            pipeline_cache.get_compute_pipeline(filter_screen_probes_pipeline_id.0),
            camera.physical_viewport_size,
        ) else {
            return Ok(());
        };

        let width = (viewport.x + 7) / 8;
        let height = (viewport.y + 7) / 8;

        {
            let command_encoder = render_context.command_encoder();
            let mut solari_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("solari_pass"),
            });
            solari_pass.set_bind_group(0, &scene_bind_group, &[]);
            solari_pass.set_bind_group(1, &bind_group.0, &[view_uniform_offset.offset]);

            solari_pass.set_pipeline(gm_buffer_pipeline);
            solari_pass.dispatch_workgroups(width, height, 1);

            solari_pass.set_pipeline(update_screen_probes_pipeline);
            solari_pass.dispatch_workgroups(width, height, 1);

            solari_pass.set_pipeline(filter_screen_probes_pipeline);
            solari_pass.dispatch_workgroups(width, height, 1);
        }

        Ok(())
    }

    fn update(&mut self, world: &mut World) {
        self.0.update_archetypes(world);
    }
}

impl FromWorld for SolariNode {
    fn from_world(world: &mut World) -> Self {
        Self(QueryState::new(world))
    }
}
