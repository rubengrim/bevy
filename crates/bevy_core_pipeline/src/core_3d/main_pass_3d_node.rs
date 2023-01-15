use crate::{
    clear_color::{ClearColor, ClearColorConfig},
    core_3d::{AlphaMask3d, Camera3d, Opaque3d, Transparent3d},
    prepass::{DepthPrepass, NormalPrepass},
};
use bevy_ecs::prelude::*;
use bevy_render::{
    camera::ExtractedCamera,
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
    render_phase::RenderPhase,
    render_resource::{
        LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
        RenderPassDescriptor,
    },
    renderer::RenderContext,
    view::{ExtractedView, ViewDepthTexture, ViewTarget},
};
#[cfg(feature = "trace")]
use bevy_utils::tracing::info_span;

use super::{Camera3dDepthLoadOp, MainPass3dTexture};

pub struct MainPass3dNode {
    query: QueryState<
        (
            &'static ExtractedCamera,
            &'static RenderPhase<Opaque3d>,
            &'static RenderPhase<AlphaMask3d>,
            &'static RenderPhase<Transparent3d>,
            &'static Camera3d,
            &'static ViewTarget,
            &'static ViewDepthTexture,
            Option<&'static MainPass3dTexture>,
            Option<&'static DepthPrepass>,
            Option<&'static NormalPrepass>,
        ),
        With<ExtractedView>,
    >,
}

impl MainPass3dNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl Node for MainPass3dNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(MainPass3dNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let Ok((
            camera,
            opaque_phase,
            alpha_mask_phase,
            transparent_phase,
            camera_3d,
            target,
            depth,
            main_pass_3d_texture,
            depth_prepass,
            normal_prepass,
        )) = self.query.get_manual(world, view_entity) else {
            // No window
            return Ok(());
        };

        let viewport = if main_pass_3d_texture.is_some() {
            None
        } else {
            camera.viewport.as_ref()
        };

        // Always run opaque pass to ensure screen is cleared
        {
            // Run the opaque pass, sorted front-to-back
            // NOTE: Scoped to drop the mutable borrow of render_context
            #[cfg(feature = "trace")]
            let _main_opaque_pass_3d_span = info_span!("main_opaque_pass_3d").entered();

            // NOTE: The opaque pass loads the color
            // buffer as well as writing to it.
            let ops = Operations {
                load: match camera_3d.clear_color {
                    ClearColorConfig::Default => {
                        LoadOp::Clear(world.resource::<ClearColor>().0.into())
                    }
                    ClearColorConfig::Custom(color) => LoadOp::Clear(color.into()),
                    ClearColorConfig::None => LoadOp::Load,
                },
                store: true,
            };
            let attachment = match main_pass_3d_texture {
                Some(MainPass3dTexture { texture }) => RenderPassColorAttachment {
                    view: &texture.default_view,
                    resolve_target: None, // TODO
                    ops,
                },
                None => target.get_color_attachment(ops),
            };

            let pass_descriptor = RenderPassDescriptor {
                label: Some("main_opaque_pass_3d"),
                color_attachments: &[Some(attachment)],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    // NOTE: The opaque main pass loads the depth buffer and possibly overwrites it
                    depth_ops: Some(Operations {
                        load: if depth_prepass.is_some() || normal_prepass.is_some() {
                            // if any prepass runs, it will generate a depth buffer so we should use it,
                            // even if only the normal_prepass is used.
                            Camera3dDepthLoadOp::Load
                        } else {
                            // NOTE: 0.0 is the far plane due to bevy's use of reverse-z projections.
                            camera_3d.depth_load_op.clone()
                        }
                        .into(),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            };

            opaque_phase.render(
                world,
                render_context,
                view_entity,
                viewport,
                pass_descriptor,
            );
        }

        // NOTE: The transparent/alpha_mask pass loads the color buffer as well as overwriting it where appropriate.
        let ops = Operations {
            load: LoadOp::Load,
            store: true,
        };
        let alpha_attachment = match main_pass_3d_texture {
            Some(MainPass3dTexture { texture }) => RenderPassColorAttachment {
                view: &texture.default_view,
                resolve_target: None, // TODO
                ops,
            },
            None => target.get_color_attachment(ops),
        };

        if !alpha_mask_phase.items.is_empty() {
            // Run the alpha mask pass, sorted front-to-back
            // NOTE: Scoped to drop the mutable borrow of render_context
            #[cfg(feature = "trace")]
            let _main_alpha_mask_pass_3d_span = info_span!("main_alpha_mask_pass_3d").entered();
            let pass_descriptor = RenderPassDescriptor {
                label: Some("main_alpha_mask_pass_3d"),
                color_attachments: &[Some(alpha_attachment.clone())],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    // NOTE: The alpha mask pass loads the depth buffer and possibly overwrites it
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            };

            alpha_mask_phase.render(
                world,
                render_context,
                view_entity,
                viewport,
                pass_descriptor,
            );
        }

        if !transparent_phase.items.is_empty() {
            // Run the transparent pass, sorted back-to-front
            // NOTE: Scoped to drop the mutable borrow of render_context
            #[cfg(feature = "trace")]
            let _main_transparent_pass_3d_span = info_span!("main_transparent_pass_3d").entered();
            let pass_descriptor = RenderPassDescriptor {
                label: Some("main_transparent_pass_3d"),
                color_attachments: &[Some(alpha_attachment)],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    // NOTE: For the transparent pass we load the depth buffer. There should be no
                    // need to write to it, but store is set to `true` as a workaround for issue #3776,
                    // https://github.com/bevyengine/bevy/issues/3776
                    // so that wgpu does not clear the depth buffer.
                    // As the opaque and alpha mask passes run first, opaque meshes can occlude
                    // transparent ones.
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            };

            transparent_phase.render(
                world,
                render_context,
                view_entity,
                viewport,
                pass_descriptor,
            );
        }

        // WebGL2 quirk: if ending with a render pass with a custom viewport, the viewport isn't
        // reset for the next render pass so add an empty render pass without a custom viewport
        #[cfg(feature = "webgl")]
        if viewport.is_some() {
            #[cfg(feature = "trace")]
            let _reset_viewport_pass_3d = info_span!("reset_viewport_pass_3d").entered();
            let pass_descriptor = RenderPassDescriptor {
                label: Some("reset_viewport_pass_3d"),
                color_attachments: &[Some(target.get_color_attachment(Operations {
                    load: LoadOp::Load,
                    store: true,
                }))],
                depth_stencil_attachment: None,
            };

            render_context
                .command_encoder
                .begin_render_pass(&pass_descriptor);
        }

        Ok(())
    }
}
