use super::{
    camera::PreviousViewProjectionUniformOffset,
    filter_screen_probes::SolariFilterScreenProbesPipelineId,
    gmt_buffer::SolariGmtBufferPipelineId, resources::SolariBindGroup,
    shade_view_target::SolariShadeViewTargetPipelineId, taa::SolariTaaPipelineId,
    update_screen_probes::SolariUpdateScreenProbesPipelineId,
    world_cache::resources::SolariWorldCacheResources,
};
use crate::scene::bind_group::SolariSceneBindGroup;
use bevy_ecs::{query::QueryItem, world::World};
use bevy_render::{
    camera::ExtractedCamera,
    render_graph::{NodeRunError, RenderGraphContext, ViewNode},
    render_resource::{ComputePassDescriptor, PipelineCache},
    renderer::RenderContext,
    view::ViewUniformOffset,
};

#[derive(Default)]
pub struct SolariNode;

impl ViewNode for SolariNode {
    type ViewQuery = (
        &'static SolariBindGroup,
        &'static SolariGmtBufferPipelineId,
        &'static SolariUpdateScreenProbesPipelineId,
        &'static SolariFilterScreenProbesPipelineId,
        &'static SolariShadeViewTargetPipelineId,
        &'static SolariTaaPipelineId,
        &'static ViewUniformOffset,
        &'static PreviousViewProjectionUniformOffset,
        &'static ExtractedCamera,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (
            bind_group,
            gmt_buffer_pipeline_id,
            update_screen_probes_pipeline_id,
            filter_screen_probes_pipeline_id,
            shade_view_target_pipeline_id,
            taa_pipeline_id,
            view_uniform_offset,
            previous_view_projection_uniform_offset,
            camera,
        ): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (
            Some(pipeline_cache),
            Some(SolariSceneBindGroup(Some(scene_bind_group))),
            Some(world_cache_resources),
        ) = (
            world.get_resource::<PipelineCache>(),
            world.get_resource::<SolariSceneBindGroup>(),
            world.get_resource::<SolariWorldCacheResources>(),
        ) else {
            return Ok(());
        };
        let (
            Some(gmt_buffer_pipeline),
            Some(update_screen_probes_pipeline),
            Some(filter_screen_probes_pipeline),
            Some(shade_view_target_pipeline),
            Some(taa_pipeline),
            Some(viewport),
        ) = (
            pipeline_cache.get_compute_pipeline(gmt_buffer_pipeline_id.0),
            pipeline_cache.get_compute_pipeline(update_screen_probes_pipeline_id.0),
            pipeline_cache.get_compute_pipeline(filter_screen_probes_pipeline_id.0),
            pipeline_cache.get_compute_pipeline(shade_view_target_pipeline_id.0),
            pipeline_cache.get_compute_pipeline(taa_pipeline_id.0),
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
            let view_dynamic_offsets = [
                view_uniform_offset.offset,
                previous_view_projection_uniform_offset.offset,
            ];
            solari_pass.set_bind_group(1, &bind_group.0, &view_dynamic_offsets);
            solari_pass.set_bind_group(2, &world_cache_resources.bind_group, &[]);

            solari_pass.set_pipeline(gmt_buffer_pipeline);
            solari_pass.dispatch_workgroups(width, height, 1);

            solari_pass.set_pipeline(update_screen_probes_pipeline);
            solari_pass.dispatch_workgroups(width, height, 1);

            solari_pass.set_pipeline(filter_screen_probes_pipeline);
            solari_pass.dispatch_workgroups(width, height, 1);

            solari_pass.set_pipeline(shade_view_target_pipeline);
            solari_pass.dispatch_workgroups(width, height, 1);

            solari_pass.set_pipeline(taa_pipeline);
            // TODO: Enable TAA
            // solari_pass.dispatch_workgroups(width, height, 1);
        }

        Ok(())
    }
}
