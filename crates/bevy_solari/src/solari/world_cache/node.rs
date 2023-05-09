use super::{
    pipelines::SolariWorldCachePipelineIds, resources::SolariWorldCacheResources, WORLD_CACHE_SIZE,
};
use bevy_ecs::world::{FromWorld, World};
use bevy_render::{
    render_graph::{Node, NodeRunError, RenderGraphContext},
    render_resource::{ComputePassDescriptor, PipelineCache},
    renderer::RenderContext,
};

pub struct SolariWorldCacheNode;

impl Node for SolariWorldCacheNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (
            Some(pipeline_cache),
            Some(world_cache_resources),
            Some(world_cache_pipeline_ids),
        ) = (
            world.get_resource::<PipelineCache>(),
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
        ) = (
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.decay_world_cache_cells),
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.compact_world_cache_single_block),
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.compact_world_cache_blocks),
            pipeline_cache.get_compute_pipeline(world_cache_pipeline_ids.compact_world_cache_write_active_cells),
        ) else {
            return Ok(());
        };

        {
            let command_encoder = render_context.command_encoder();
            let mut solari_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("solari_update_world_cache_pass"),
            });
            solari_pass.set_bind_group(0, &world_cache_resources.bind_group, &[]);

            solari_pass.set_pipeline(decay_world_cache_cells_pipeline);
            solari_pass.dispatch_workgroups((WORLD_CACHE_SIZE / 1024) as u32, 1, 1);

            solari_pass.set_pipeline(compact_world_cache_single_block_pipeline);
            solari_pass.dispatch_workgroups((WORLD_CACHE_SIZE / 1024) as u32, 1, 1);

            solari_pass.set_pipeline(compact_world_cache_blocks_pipeline);
            solari_pass.dispatch_workgroups(1, 1, 1);

            solari_pass.set_pipeline(compact_world_cache_write_active_cells_pipeline);
            solari_pass.dispatch_workgroups((WORLD_CACHE_SIZE / 1024) as u32, 1, 1);
        }

        Ok(())
    }

    fn update(&mut self, _world: &mut World) {}
}

impl FromWorld for SolariWorldCacheNode {
    fn from_world(_world: &mut World) -> Self {
        Self
    }
}
