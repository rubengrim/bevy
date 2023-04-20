use crate::{
    misc::{create_view_bind_group, SolariTextures},
    pipeline::{SampleCount, SolariPipeline, SolariPipelineId},
    scene::SceneBindGroup,
};
use bevy_ecs::{
    query::QueryState,
    world::{FromWorld, World},
};
use bevy_render::{
    camera::ExtractedCamera,
    render_graph::{Node, NodeRunError, RenderGraphContext},
    render_resource::{ComputePassDescriptor, PipelineCache},
    renderer::RenderContext,
    view::{ViewTarget, ViewUniformOffset, ViewUniforms},
};
use std::sync::atomic::Ordering;

pub struct SolariNode {
    view_query: QueryState<(
        &'static SolariPipelineId,
        &'static SolariTextures,
        &'static ViewTarget,
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
            Ok((pipeline_id, textures, view_target, view_uniform_offset, camera)),
            Some(pipeline_cache),
            Some(SceneBindGroup(Some(scene_bind_group))),
            Some(view_uniforms),
            Some(solari_pipeline),
            Some(sample_count),
        ) = (
            self.view_query.get_manual(world, graph.view_entity()),
            world.get_resource::<PipelineCache>(),
            world.get_resource::<SceneBindGroup>(),
            world.get_resource::<ViewUniforms>(),
            world.get_resource::<SolariPipeline>(),
            world.get_resource::<SampleCount>()
        ) else {
            return Ok(());
        };
        let (Some(pipeline), Some(viewport)) = (pipeline_cache.get_compute_pipeline(pipeline_id.0), camera.physical_viewport_size) else {
            return Ok(());
        };
        let Some(view_bind_group) = create_view_bind_group(view_uniforms, view_target, textures, solari_pipeline, render_context.render_device()) else {
            return Ok(());
        };

        let previous_sample_count = sample_count.0.fetch_add(1, Ordering::SeqCst) as f32;

        {
            let mut solari_pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("solari_pass"),
                    });

            solari_pass.set_pipeline(pipeline);
            solari_pass.set_bind_group(0, scene_bind_group, &[]);
            solari_pass.set_bind_group(1, &view_bind_group, &[view_uniform_offset.offset]);
            solari_pass.set_push_constants(0, &previous_sample_count.to_le_bytes());
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
