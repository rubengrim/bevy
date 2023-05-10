use super::{
    camera::SolariPathTracer,
    pipeline::{SolariPathTracerPipelineId, SolariPathtracerPipeline, TraceRaysFromBufferId},
    resources::{create_view_bind_group, SolariPathTracerAccumulationTexture},
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
    view::{ViewTarget, ViewUniformOffset, ViewUniforms},
};
use std::sync::atomic::Ordering;

pub struct SolariPathTracerNode(
    QueryState<(
        &'static SolariPathTracer,
        &'static SolariPathTracerPipelineId,
        &'static TraceRaysFromBufferId,
        &'static SolariPathTracerAccumulationTexture,
        &'static ViewTarget,
        &'static ViewUniformOffset,
        &'static ExtractedCamera,
    )>,
);

impl Node for SolariPathTracerNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (
            Ok((path_tracer, pipeline_id,tr_id, accumulation_texture, view_target, view_uniform_offset, camera)),
            Some(pipeline_cache),
            Some(SolariSceneBindGroup(Some(scene_bind_group))),
            Some(view_uniforms),
            Some(solari_pipeline),
        ) = (
            self.0.get_manual(world, graph.view_entity()),
            world.get_resource::<PipelineCache>(),
            world.get_resource::<SolariSceneBindGroup>(),
            world.get_resource::<ViewUniforms>(),
            world.get_resource::<SolariPathtracerPipeline>(),
        ) else {
            return Ok(());
        };
        let (Some(pipeline), Some(tr_pipeline), Some(viewport)) = (pipeline_cache.get_compute_pipeline(pipeline_id.0), pipeline_cache.get_compute_pipeline(tr_id.0), camera.physical_viewport_size) else {
            return Ok(());
        };
        let Some(view_bind_group1) = create_view_bind_group(view_uniforms, accumulation_texture, view_target, solari_pipeline, render_context.render_device(), false) else {
            return Ok(());
        };
        let Some(view_bind_group2) = create_view_bind_group(view_uniforms, accumulation_texture, view_target, solari_pipeline, render_context.render_device(), true) else {
            return Ok(());
        };

        let previous_sample_count = path_tracer.sample_count.fetch_add(1, Ordering::SeqCst) as f32;

        {
            let command_encoder = render_context.command_encoder();
            let mut solari_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("solari_path_tracer_pass"),
            });
            solari_pass.set_bind_group(0, &scene_bind_group, &[]);
            solari_pass.set_bind_group(1, &view_bind_group1, &[view_uniform_offset.offset]);

            solari_pass.set_pipeline(pipeline);
            solari_pass.set_push_constants(0, &previous_sample_count.to_le_bytes());
            solari_pass.dispatch_workgroups((viewport.x + 7) / 8, (viewport.y + 7) / 8, 1);

            solari_pass.set_pipeline(tr_pipeline);
            solari_pass.dispatch_workgroups((viewport.x + 7) / 8, (viewport.y + 7) / 8, 1);
        }

        Ok(())
    }

    fn update(&mut self, world: &mut World) {
        self.0.update_archetypes(world);
    }
}

impl FromWorld for SolariPathTracerNode {
    fn from_world(world: &mut World) -> Self {
        Self(QueryState::new(world))
    }
}
