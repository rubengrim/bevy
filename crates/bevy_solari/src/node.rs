use crate::{
    material::MaterialBuffer,
    misc::create_view_bind_group,
    pipeline::{SolariPipeline, SolariPipelineId},
    tlas::TlasResource,
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

pub struct SolariNode {
    view_query: QueryState<(
        &'static SolariPipelineId,
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
            Ok((pipeline_id, view_target, view_uniform_offset, camera)),
            Some(pipeline_cache),
            Some(view_uniforms),
            Some(tlas),
            Some(solari_pipeline),
            Some(material_buffer),
        ) = (
            self.view_query.get_manual(world, graph.view_entity()),
            world.get_resource::<PipelineCache>(),
            world.get_resource::<ViewUniforms>(),
            world.get_resource::<TlasResource>(),
            world.get_resource::<SolariPipeline>(),
            world.get_resource::<MaterialBuffer>(),
        ) else {
            return Ok(());
        };
        let (Some(pipeline), Some(viewport)) = (pipeline_cache.get_compute_pipeline(pipeline_id.0), camera.physical_viewport_size) else {
            return Ok(());
        };
        let Some(view_bind_group) = create_view_bind_group(view_target, view_uniforms, tlas, solari_pipeline, material_buffer, render_context.render_device()) else {
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
            solari_pass.set_bind_group(0, &view_bind_group, &[view_uniform_offset.offset]);
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
