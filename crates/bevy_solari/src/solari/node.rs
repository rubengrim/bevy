use super::{
    camera::PreviousViewProjectionUniformOffset,
    pipelines::SolariPipelineIds,
    view_resources::SolariViewBindGroup,
    world_cache::{
        pipelines::SolariWorldCachePipelineIds,
        resources::{SolariWorldCacheResources, WORLD_CACHE_SIZE},
    },
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
        &'static SolariViewBindGroup,
        &'static SolariPipelineIds,
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
            pipeline_ids,
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
            Some(world_cache_pipeline_ids),
        ) = (
            world.get_resource::<PipelineCache>(),
            world.get_resource::<SolariSceneBindGroup>(),
            world.get_resource::<SolariWorldCacheResources>(),
            world.get_resource::<SolariWorldCachePipelineIds>(),
        ) else {
            return Ok(());
        };
        let (
            Some(decay_world_cache_cells_pipeline),
            Some(compact_world_cache_single_block_pipeline),
            Some(compact_world_cache_blocks_pipeline),
            Some(compact_world_cache_write_active_cells_pipeline),
            Some(world_cache_sample_irradiance_pipeline),
            Some(world_cache_blend_new_samples_pipeline),
            Some(gmt_buffer_pipeline),
            Some(update_screen_probes_pipeline),
            Some(filter_screen_probes_pipeline),
            Some(intepolate_screen_probes_pipeline),
            Some(denoise_indirect_diffuse_temporal_pipeline),
            Some(denoise_indirect_diffuse_spatial_pipeline),
            Some(sample_direct_diffuse_pipeline),
            Some(denoise_direct_diffuse_temporal_pipeline),
            Some(denoise_direct_diffuse_spatial_pipeline),
            Some(shade_view_target_pipeline),
            Some(_taa_pipeline),
            Some(viewport),
        ) = (
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.decay_world_cache_cells),
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.compact_world_cache_single_block),
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.compact_world_cache_blocks),
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.compact_world_cache_write_active_cells),
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.world_cache_sample_irradiance),
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.world_cache_blend_new_samples),
            pipeline_cache.get_compute_pipeline(pipeline_ids.gmt_buffer),
            pipeline_cache.get_compute_pipeline(pipeline_ids.update_screen_probes),
            pipeline_cache.get_compute_pipeline(pipeline_ids.filter_screen_probes),
            pipeline_cache.get_compute_pipeline(pipeline_ids.interpolate_screen_probes),
            pipeline_cache.get_compute_pipeline(pipeline_ids.denoise_indirect_diffuse_temporal),
            pipeline_cache.get_compute_pipeline(pipeline_ids.denoise_indirect_diffuse_spatial),
            pipeline_cache.get_compute_pipeline(pipeline_ids.sample_direct_diffuse),
            pipeline_cache.get_compute_pipeline(pipeline_ids.denoise_direct_diffuse_temporal),
            pipeline_cache.get_compute_pipeline(pipeline_ids.denoise_direct_diffuse_spatial),
            pipeline_cache.get_compute_pipeline(pipeline_ids.shade_view_target),
            pipeline_cache.get_compute_pipeline(pipeline_ids.taa),
            camera.physical_viewport_size,
        ) else {
            return Ok(());
        };

        let width = (viewport.x + 7) / 8;
        let height = (viewport.y + 7) / 8;
        let view_dynamic_offsets = [
            view_uniform_offset.offset,
            previous_view_projection_uniform_offset.offset,
        ];

        let command_encoder = render_context.command_encoder();
        let mut solari_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("solari_pass"),
        });

        solari_pass.push_debug_group("world_cache_update");

        solari_pass.set_bind_group(0, &world_cache_resources.bind_group, &[]);

        solari_pass.set_pipeline(decay_world_cache_cells_pipeline);
        solari_pass.dispatch_workgroups((WORLD_CACHE_SIZE / 1024) as u32, 1, 1);

        solari_pass.set_pipeline(compact_world_cache_single_block_pipeline);
        solari_pass.dispatch_workgroups((WORLD_CACHE_SIZE / 1024) as u32, 1, 1);

        solari_pass.set_pipeline(compact_world_cache_blocks_pipeline);
        solari_pass.dispatch_workgroups(1, 1, 1);

        solari_pass.set_pipeline(compact_world_cache_write_active_cells_pipeline);
        solari_pass.dispatch_workgroups((WORLD_CACHE_SIZE / 1024) as u32, 1, 1);

        solari_pass.set_bind_group(0, &scene_bind_group, &[]);
        solari_pass.set_bind_group(1, &bind_group.0, &view_dynamic_offsets);
        solari_pass.set_bind_group(2, &world_cache_resources.bind_group_no_dispatch, &[]);

        solari_pass.set_pipeline(world_cache_sample_irradiance_pipeline);
        solari_pass
            .dispatch_workgroups_indirect(&world_cache_resources.active_cells_dispatch_buffer, 0);

        solari_pass.set_pipeline(world_cache_blend_new_samples_pipeline);
        solari_pass
            .dispatch_workgroups_indirect(&world_cache_resources.active_cells_dispatch_buffer, 0);

        solari_pass.pop_debug_group();

        solari_pass.push_debug_group("screen_shading");

        let mut dispatch = |pipeline| {
            solari_pass.set_pipeline(pipeline);
            solari_pass.dispatch_workgroups(width, height, 1);
        };

        dispatch(gmt_buffer_pipeline);
        dispatch(update_screen_probes_pipeline);
        dispatch(filter_screen_probes_pipeline);
        dispatch(intepolate_screen_probes_pipeline);
        dispatch(denoise_indirect_diffuse_temporal_pipeline);
        dispatch(denoise_indirect_diffuse_spatial_pipeline);
        dispatch(sample_direct_diffuse_pipeline);
        dispatch(denoise_direct_diffuse_temporal_pipeline);
        dispatch(denoise_direct_diffuse_spatial_pipeline);
        dispatch(shade_view_target_pipeline);
        // TODO: Enable TAA
        // dispatch(taa_pipeline);

        solari_pass.pop_debug_group();

        Ok(())
    }
}
