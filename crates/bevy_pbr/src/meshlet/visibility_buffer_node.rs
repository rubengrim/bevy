use super::{gpu_scene::MeshletGpuScene, pipelines::MeshletPipelines};
use bevy_core_pipeline::core_3d::{Camera3d, Camera3dDepthLoadOp};
use bevy_ecs::{query::QueryItem, world::World};
use bevy_render::{
    camera::ExtractedCamera,
    color::Color,
    render_graph::{NodeRunError, RenderGraphContext, ViewNode},
    render_resource::{
        ComputePassDescriptor, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
        RenderPassDepthStencilAttachment, RenderPassDescriptor,
    },
    renderer::RenderContext,
    view::{ViewDepthTexture, ViewUniformOffset},
};

pub mod draw_3d_graph {
    pub mod node {
        pub const MESHLET_VISIBILITY_BUFFER_PASS: &str = "meshlet_visibility_buffer_pass";
    }
}

#[derive(Default)]
pub struct MeshletVisibilityBufferPassNode;
impl ViewNode for MeshletVisibilityBufferPassNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static Camera3d,
        &'static ViewDepthTexture,
        &'static ViewUniformOffset,
    );

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (camera, camera_3d, depth, view_offset): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (Some(gpu_scene), Some((Some(culling_pipeline), Some(visibility_buffer_pipeline)))) = (
            world.get_resource::<MeshletGpuScene>(),
            MeshletPipelines::get(world),
        ) else {
            return Ok(());
        };
        let (
            scene_meshlet_count,
            Some(culling_bind_group),
            Some(visibility_buffer_bind_group),
            Some(draw_index_buffer),
            Some(draw_command_buffer),
            Some(visibility_buffer),
        ) = gpu_scene.visibility_buffer_resources_for_view(&graph.view_entity())
        else {
            return Ok(());
        };

        let depth_load = if depth.is_first_write() {
            camera_3d.depth_load_op.clone()
        } else {
            Camera3dDepthLoadOp::Load
        }
        .into();

        {
            let command_encoder = render_context.command_encoder();
            let mut culling_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("meshlet_culling_pass"),
            });
            culling_pass.set_bind_group(0, culling_bind_group, &[view_offset.offset]);
            culling_pass.set_pipeline(culling_pipeline);
            culling_pass.dispatch_workgroups((scene_meshlet_count + 127) / 128, 1, 1);
        }

        {
            let mut draw_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some(draw_3d_graph::node::MESHLET_VISIBILITY_BUFFER_PASS),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: visibility_buffer,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK.into()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    depth_ops: Some(Operations {
                        load: depth_load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            if let Some(viewport) = camera.viewport.as_ref() {
                draw_pass.set_camera_viewport(viewport);
            }

            draw_pass.set_index_buffer(draw_index_buffer.slice(..), 0, IndexFormat::Uint32);
            draw_pass.set_bind_group(0, visibility_buffer_bind_group, &[view_offset.offset]);
            draw_pass.set_render_pipeline(visibility_buffer_pipeline);
            draw_pass.draw_indexed_indirect(draw_command_buffer, 0);
        }

        Ok(())
    }
}
