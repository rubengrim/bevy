pub use bevy_render::{DlssAvailable, DlssProjectId};
pub use dlss_wgpu::DlssPreset;

use crate::{
    core_3d::ViewportOverride,
    prelude::Camera3d,
    prepass::{DepthPrepass, MotionVectorPrepass, ViewPrepassTextures},
};
use bevy_app::{App, Plugin};
use bevy_core::FrameCount;
use bevy_ecs::{
    prelude::{Bundle, Component, Entity, NonSendMut, Query},
    query::{QueryState, With},
    schedule::IntoSystemConfigs,
    system::{Commands, Res, ResMut},
    world::{FromWorld, World},
};
use bevy_math::{UVec2, Vec4Swizzles};
use bevy_render::{
    camera::{ExtractedCamera, MipBias, TemporalJitter, Viewport},
    prelude::{Camera, Msaa, Projection},
    render_graph::{Node, NodeRunError, RenderGraphApp, RenderGraphContext},
    render_resource::{
        CommandEncoder, CommandEncoderDescriptor, ImageSubresourceRange, TextureAspect,
        TextureUsages,
    },
    renderer::{RenderAdapter, RenderContext, RenderDevice, RenderQueue},
    view::{prepare_view_uniforms, ExtractedView, ViewTarget},
    ExtractSchedule, MainWorld, Render, RenderApp, RenderSet,
};
use bevy_utils::{tracing::info, HashMap};
use dlss_wgpu::{
    DlssContext, DlssExposure, DlssFeatureFlags, DlssRenderParameters, DlssSdk, DlssTexture,
};
use std::{
    mem,
    rc::Rc,
    sync::{Mutex, MutexGuard},
};

mod draw_3d_graph {
    pub mod node {
        /// Label for the DLSS render node.
        pub const DLSS: &str = "dlss";
    }
}

pub struct DlssPlugin;

impl Plugin for DlssPlugin {
    fn build(&self, app: &mut App) {
        if app.get_sub_app_mut(RenderApp).is_err() {
            return;
        }
        if app.world.get_resource::<DlssAvailable>().is_none() {
            info!("DLSS not available");
            return;
        }

        let project_id = app.world.resource::<DlssProjectId>().0;
        let render_device = app
            .get_sub_app_mut(RenderApp)
            .unwrap()
            .world
            .resource::<RenderDevice>()
            .clone();

        let dlss_sdk = DlssSdk::new(project_id, render_device);
        if dlss_sdk.is_err() {
            app.world.remove_resource::<DlssAvailable>();
            info!("DLSS not available");
            return;
        }

        app.insert_resource(Msaa::Off);

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .insert_non_send_resource(DlssResource {
                sdk: dlss_sdk.unwrap(),
                context_cache: HashMap::new(),
            })
            .add_systems(ExtractSchedule, extract_dlss_settings)
            .add_systems(
                Render,
                prepare_dlss
                    .in_set(RenderSet::Prepare)
                    .before(prepare_view_uniforms),
            )
            .add_render_graph_node::<DlssNode>(
                crate::core_3d::graph::NAME,
                draw_3d_graph::node::DLSS,
            )
            .add_render_graph_edges(
                crate::core_3d::graph::NAME,
                &[
                    crate::core_3d::graph::node::END_MAIN_PASS,
                    draw_3d_graph::node::DLSS,
                    crate::core_3d::graph::node::BLOOM,
                    crate::core_3d::graph::node::TONEMAPPING,
                ],
            );
    }
}

pub struct DlssNode {
    view_query: QueryState<(
        &'static ExtractedView,
        &'static DlssSettings,
        &'static ViewportOverride,
        &'static TemporalJitter,
        &'static ViewTarget,
        &'static ViewPrepassTextures,
    )>,
}

impl FromWorld for DlssNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            view_query: QueryState::new(world),
        }
    }
}

impl Node for DlssNode {
    fn update(&mut self, world: &mut World) {
        self.view_query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let adapter = world.resource::<RenderAdapter>();
        let dlss = world.non_send_resource::<DlssResource>();
        let Ok((view, dlss_settings, viewport_override, temporal_jitter, view_target, prepass_textures))
            = self.view_query.get_manual(world, graph.view_entity()) else { return Ok(()); };
        let (
            Some(prepass_motion_vectors_texture),
            Some(prepass_depth_texture),
        ) = (
            &prepass_textures.motion_vectors,
            &prepass_textures.depth,
        ) else {
            return Ok(());
        };
        let render_resolution = viewport_override.0.physical_size;
        let upscaled_resolution = view.viewport.zw();
        let mut dlss_context = dlss.get_context(upscaled_resolution, dlss_settings.preset);
        let view_target = view_target.post_process_write();

        dlss_context
            .render(
                DlssRenderParameters {
                    color: DlssTexture {
                        texture: &view_target.source_texture,
                        view: &view_target.source,
                        subresource_range: ImageSubresourceRange {
                            aspect: TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                        usages: TextureUsages::TEXTURE_BINDING,
                    },
                    depth: DlssTexture {
                        texture: &prepass_depth_texture.texture,
                        view: &prepass_depth_texture.default_view,
                        subresource_range: ImageSubresourceRange {
                            aspect: TextureAspect::DepthOnly,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                        usages: TextureUsages::TEXTURE_BINDING,
                    },
                    motion_vectors: DlssTexture {
                        texture: &prepass_motion_vectors_texture.texture,
                        view: &prepass_motion_vectors_texture.default_view,
                        subresource_range: ImageSubresourceRange {
                            aspect: TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                        usages: TextureUsages::TEXTURE_BINDING,
                    },
                    exposure: DlssExposure::Automatic,
                    transparency_mask: None, // TODO
                    bias: None,              // TODO
                    dlss_output: DlssTexture {
                        texture: &view_target.destination_texture,
                        view: &view_target.destination,
                        subresource_range: ImageSubresourceRange {
                            aspect: TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                        usages: TextureUsages::STORAGE_BINDING,
                    },
                    reset: dlss_settings.reset,
                    jitter_offset: temporal_jitter.offset,
                    partial_texture_size: Some(render_resolution),
                    motion_vector_scale: Some(render_resolution.as_vec2()),
                },
                render_context.command_encoder(),
                &adapter.0,
            )
            .expect("Failed to render DLSS");

        Ok(())
    }
}

#[derive(Bundle, Default)]
pub struct DlssBundle {
    pub settings: DlssSettings,
    pub jitter: TemporalJitter,
    pub depth_prepass: DepthPrepass,
    pub motion_vector_prepass: MotionVectorPrepass,
}

#[derive(Component, Clone, Default)]
pub struct DlssSettings {
    pub preset: DlssPreset,
    pub reset: bool,
}

struct DlssResource {
    sdk: Rc<DlssSdk<RenderDevice>>,
    context_cache: HashMap<(UVec2, DlssPreset), (Mutex<DlssContext<RenderDevice>>, bool)>,
}

impl DlssResource {
    fn get_or_create_context(
        &mut self,
        upscaled_resolution: UVec2,
        dlss_preset: DlssPreset,
        hdr: bool,
        maybe_command_encoder: &mut Option<CommandEncoder>,
        render_device: &RenderDevice,
    ) -> MutexGuard<DlssContext<RenderDevice>> {
        let dlss_sdk = Rc::clone(&self.sdk);

        let mut dlss_context = self
            .context_cache
            .entry((upscaled_resolution, dlss_preset))
            .or_insert_with(|| {
                if maybe_command_encoder.is_none() {
                    *maybe_command_encoder = Some(render_device.create_command_encoder(
                        &CommandEncoderDescriptor {
                            label: Some("dlss_context_creation_command_encoder"),
                        },
                    ));
                }

                let mut dlss_feature_flags = DlssFeatureFlags::LowResolutionMotionVectors
                    | DlssFeatureFlags::InvertedDepth
                    | DlssFeatureFlags::AutoExposure
                    | DlssFeatureFlags::PartialTextureInputs;
                if hdr {
                    dlss_feature_flags |= DlssFeatureFlags::HighDynamicRange;
                }

                let dlss_context = DlssContext::new(
                    upscaled_resolution,
                    dlss_preset,
                    dlss_feature_flags,
                    &dlss_sdk,
                    maybe_command_encoder.as_mut().unwrap(),
                )
                .expect("Failed to create DlssContext");

                (Mutex::new(dlss_context), true)
            });

        dlss_context.1 = true;
        dlss_context.0.lock().unwrap()
    }

    fn get_context(
        &self,
        upscaled_resolution: UVec2,
        dlss_preset: DlssPreset,
    ) -> MutexGuard<DlssContext<RenderDevice>> {
        self.context_cache[&(upscaled_resolution, dlss_preset)]
            .0
            .lock()
            .unwrap()
    }

    fn drop_stale_contexts(&mut self) {
        self.context_cache
            .retain(|_, (_, in_use)| mem::take(in_use));
    }
}

fn extract_dlss_settings(mut commands: Commands, mut main_world: ResMut<MainWorld>) {
    let mut query = main_world
        .query_filtered::<(Entity, &Camera, &Projection, &mut DlssSettings), (
            With<Camera3d>,
            With<TemporalJitter>,
            With<DepthPrepass>,
            With<MotionVectorPrepass>,
        )>();

    for (entity, camera, camera_projection, mut dlss_settings) in query.iter_mut(&mut main_world) {
        let has_perspective_projection = matches!(camera_projection, Projection::Perspective(_));
        if camera.is_active && has_perspective_projection {
            commands.get_or_spawn(entity).insert((
                dlss_settings.clone(),
                camera_projection.clone(),
                MipBias(0.0),
            ));

            dlss_settings.reset = false;
        }
    }
}

fn prepare_dlss(
    mut query: Query<(
        Entity,
        &ExtractedView,
        &ExtractedCamera,
        &DlssSettings,
        &mut TemporalJitter,
        &mut MipBias,
    )>,
    mut dlss: NonSendMut<DlssResource>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    frame_count: Res<FrameCount>,
    mut commands: Commands,
) {
    let mut maybe_command_encoder = None;

    for (entity, view, camera, dlss_settings, mut temporal_jitter, mut mip_bias) in &mut query {
        let upscaled_resolution = view.viewport.zw();
        let dlss_context = dlss.get_or_create_context(
            upscaled_resolution,
            dlss_settings.preset,
            view.hdr,
            &mut maybe_command_encoder,
            &render_device,
        );
        let render_resolution = dlss_context.render_resolution();

        temporal_jitter.offset = dlss_context.suggested_jitter(frame_count.0, render_resolution);
        mip_bias.0 = -1.0; // TODO

        commands.entity(entity).insert(ViewportOverride(Viewport {
            physical_position: view.viewport.xy(),
            physical_size: render_resolution,
            depth: camera.viewport.clone().map(|v| v.depth).unwrap_or(0.0..1.0),
        }));
    }

    if let Some(command_encoder) = maybe_command_encoder {
        render_queue.submit([command_encoder.finish()]);
    }

    dlss.drop_stale_contexts();
}
