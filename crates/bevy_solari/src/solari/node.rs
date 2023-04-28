use super::{
    filter_screen_probes::SolariFilterScreenProbesPipelineId, resources::SolariBindGroups,
    update_screen_probes::SolariUpdateScreenProbesPipelineId,
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
        &'static SolariBindGroups,
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
            Ok((bind_groups, update_screen_probes_pipeline_id, filter_screen_probes_pipeline_id, view_uniform_offset, camera)),
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
            Some(update_screen_probes_pipeline),
            Some(filter_screen_probes_pipeline),
            Some(viewport),
        ) = (
            pipeline_cache.get_compute_pipeline(update_screen_probes_pipeline_id.0),
            pipeline_cache.get_compute_pipeline(filter_screen_probes_pipeline_id.0),
            camera.physical_viewport_size,
        ) else {
            return Ok(());
        };

        let width = round_up_to_multiple_of_8(viewport.x);
        let height = round_up_to_multiple_of_8(viewport.y);
        let x_probes = width / 8;
        let y_probes = height / 8;

        {
            let command_encoder = render_context.command_encoder();
            let mut solari_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("solari_pass"),
            });
            solari_pass.set_bind_group(0, &scene_bind_group, &[]);

            solari_pass.set_pipeline(update_screen_probes_pipeline);
            solari_pass.set_bind_group(
                1,
                &bind_groups.update_screen_probes,
                &[view_uniform_offset.offset],
            );
            solari_pass.dispatch_workgroups(width, height, 1);

            solari_pass.set_pipeline(filter_screen_probes_pipeline);
            solari_pass.set_bind_group(1, &bind_groups.filter_screen_probes, &[]);
            solari_pass.dispatch_workgroups(x_probes, y_probes, 1);
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

fn round_up_to_multiple_of_8(x: u32) -> u32 {
    (x + 7) & !7
}
