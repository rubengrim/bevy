use crate::tonemapping::{
    TonemappingLocalBindGroups, TonemappingLocalComputeLuminancesPipeline,
    TonemappingLocalPipelineIds, TonemappingLocalTextures, TonemappingMode, TonemappingPipeline,
    TonemappingSettings, ViewTonemappingPipeline,
};
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryState;
use bevy_render::{
    camera::ExtractedCamera,
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
    render_resource::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, ComputePassDescriptor,
        PipelineCache, TextureViewId,
    },
    renderer::RenderContext,
    view::ViewTarget,
};
use std::sync::Mutex;

pub struct TonemappingNode {
    query: QueryState<(
        &'static ViewTarget,
        &'static ExtractedCamera,
        &'static TonemappingSettings,
        Option<&'static ViewTonemappingPipeline>,
        Option<&'static TonemappingLocalTextures>,
        Option<&'static TonemappingLocalBindGroups>,
        Option<&'static TonemappingLocalPipelineIds>,
    )>,
    cached_texture_bind_group: Mutex<Option<(TextureViewId, BindGroup)>>,
}

impl TonemappingNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
            cached_texture_bind_group: Mutex::new(None),
        }
    }
}

impl Node for TonemappingNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(TonemappingNode::IN_VIEW, SlotType::Entity)]
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
        let pipeline_cache = world.resource::<PipelineCache>();
        let tonemapping_pipeline = world.resource::<TonemappingPipeline>();
        let local_compute_luminances_pipeline =
            world.resource::<TonemappingLocalComputeLuminancesPipeline>();

        let (
            target,
            camera,
            tonemapping_settings,
            tonemapping,
            tonemapping_textures,
            tonemapping_bind_groups,
            tonemapping_local_pipelines,
        ) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        if !target.is_hdr() {
            return Ok(());
        }

        // let pipeline = match pipeline_cache.get_render_pipeline(tonemapping.0) {
        //     Some(pipeline) => pipeline,
        //     None => return Ok(()),
        // };

        let post_process = target.post_process_write();
        let source = post_process.source;
        let destination = post_process.destination;

        match tonemapping_settings.mode {
            TonemappingMode::Global => todo!(),
            TonemappingMode::Local => {
                let camera_size = camera.physical_viewport_size.unwrap();
                let textures = tonemapping_textures.unwrap();
                let tonemapping_bind_groups = tonemapping_bind_groups.unwrap();
                let tonemapping_local_pipelines = tonemapping_local_pipelines.unwrap();

                let compute_luminances_pipeline = pipeline_cache
                    .get_compute_pipeline(tonemapping_local_pipelines.compute_luminances)
                    .unwrap();
                let compute_weights_pipeline = pipeline_cache
                    .get_compute_pipeline(tonemapping_local_pipelines.compute_weights)
                    .unwrap();
                let weigh_exposures_pipeline = pipeline_cache
                    .get_compute_pipeline(tonemapping_local_pipelines.weigh_exposures)
                    .unwrap();
                let blend_laplacian_pipeline = pipeline_cache
                    .get_compute_pipeline(tonemapping_local_pipelines.blend_laplacian)
                    .unwrap();

                let compute_luminances_bind_group =
                    render_context
                        .render_device
                        .create_bind_group(&BindGroupDescriptor {
                            label: Some("tonemapping_local_compute_luminances_bind_group"),
                            layout: &local_compute_luminances_pipeline.bind_group_layout,
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::TextureView(source),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::TextureView(
                                        &TonemappingLocalTextures::texture_view(
                                            &textures.luminances,
                                            0,
                                        ),
                                    ),
                                },
                            ],
                        });

                {
                    let mut pass =
                        render_context
                            .command_encoder
                            .begin_compute_pass(&ComputePassDescriptor {
                                label: Some("tonemapping_local_compute_luminances_pass"),
                            });
                    pass.set_pipeline(compute_luminances_pipeline);
                    pass.set_bind_group(0, &compute_luminances_bind_group, &[]);
                    pass.dispatch_workgroups((camera_size.x + 7) / 8, (camera_size.y + 7) / 8, 1);
                }

                {
                    let mut pass =
                        render_context
                            .command_encoder
                            .begin_compute_pass(&ComputePassDescriptor {
                                label: Some("tonemapping_local_compute_weights_pass"),
                            });
                    pass.set_pipeline(compute_weights_pipeline);
                    pass.set_bind_group(0, &tonemapping_bind_groups.compute_weights, &[]);
                    pass.dispatch_workgroups(
                        (camera_size.x + 15) / 16,
                        (camera_size.y + 15) / 16,
                        1,
                    );
                }

                {
                    let mut pass =
                        render_context
                            .command_encoder
                            .begin_compute_pass(&ComputePassDescriptor {
                                label: Some("tonemapping_local_weigh_exposures_pass"),
                            });
                    pass.set_pipeline(weigh_exposures_pipeline);
                    pass.set_bind_group(0, &tonemapping_bind_groups.weigh_exposures, &[]);
                    pass.dispatch_workgroups((camera_size.x + 7) / 8, (camera_size.y + 7) / 8, 1);
                }

                {
                    let mut pass =
                        render_context
                            .command_encoder
                            .begin_compute_pass(&ComputePassDescriptor {
                                label: Some("tonemapping_local_blend_laplacian_pass"),
                            });
                    pass.set_pipeline(blend_laplacian_pipeline);
                    for i in 0..5 {
                        pass.set_bind_group(0, &tonemapping_bind_groups.blend_laplacians[i], &[]);

                        let camera_size = camera_size / 2u32.pow(4 - i as u32);
                        pass.dispatch_workgroups(
                            (camera_size.x + 7) / 8,
                            (camera_size.y + 7) / 8,
                            1,
                        );
                    }
                }

                return Ok(());
            }
        }

        // let mut cached_bind_group = self.cached_texture_bind_group.lock().unwrap();
        // let bind_group = match &mut *cached_bind_group {
        //     Some((id, bind_group)) if source.id() == *id => bind_group,
        //     cached_bind_group => {
        //         let sampler = render_context
        //             .render_device
        //             .create_sampler(&SamplerDescriptor::default());

        //         let bind_group =
        //             render_context
        //                 .render_device
        //                 .create_bind_group(&BindGroupDescriptor {
        //                     label: None,
        //                     layout: &tonemapping_pipeline.texture_bind_group,
        //                     entries: &[
        //                         BindGroupEntry {
        //                             binding: 0,
        //                             resource: BindingResource::TextureView(source),
        //                         },
        //                         BindGroupEntry {
        //                             binding: 1,
        //                             resource: BindingResource::Sampler(&sampler),
        //                         },
        //                     ],
        //                 });

        //         let (_, bind_group) = cached_bind_group.insert((source.id(), bind_group));
        //         bind_group
        //     }
        // };

        // let pass_descriptor = RenderPassDescriptor {
        //     label: Some("tonemapping_pass"),
        //     color_attachments: &[Some(RenderPassColorAttachment {
        //         view: destination,
        //         resolve_target: None,
        //         ops: Operations {
        //             load: LoadOp::Clear(Default::default()), // TODO shouldn't need to be cleared
        //             store: true,
        //         },
        //     })],
        //     depth_stencil_attachment: None,
        // };

        // let mut render_pass = render_context
        //     .command_encoder
        //     .begin_render_pass(&pass_descriptor);

        // render_pass.set_pipeline(pipeline);
        // render_pass.set_bind_group(0, bind_group, &[]);
        // render_pass.draw(0..3, 0..1);

        // Ok(())
    }
}
