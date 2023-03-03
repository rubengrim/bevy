pub use bevy_render::{DlssAvailable, DlssProjectId};
pub use dlss_wgpu::DlssPreset;

use crate::{
    core_3d::ViewportOverride,
    prelude::Camera3d,
    prepass::{DepthPrepass, MotionVectorPrepass},
};
use bevy_app::{App, IntoSystemAppConfig, Plugin};
use bevy_core::FrameCount;
use bevy_ecs::{
    prelude::{Bundle, Component, Entity, NonSendMut, Query},
    query::With,
    schedule::IntoSystemConfig,
    system::{Commands, Res, ResMut},
};
use bevy_math::{UVec2, Vec4Swizzles};
use bevy_render::{
    camera::{TemporalJitter, Viewport},
    prelude::{Camera, Msaa, Projection},
    render_resource::{CommandEncoder, CommandEncoderDescriptor},
    renderer::{RenderDevice, RenderQueue},
    view::ExtractedView,
    ExtractSchedule, MainWorld, RenderApp, RenderSet,
};
use bevy_utils::{tracing::info, HashMap};
use dlss_wgpu::{DlssContext, DlssFeatureFlags, DlssSdk};
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
            .add_system(extract_dlss_settings.in_schedule(ExtractSchedule))
            .add_system(
                prepare_dlss
                    .in_set(RenderSet::Prepare)
                    .before(prepare_view_uniforms),
            );

        let dlss_node = DlssNode::new(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let draw_3d_graph = graph
            .get_sub_graph_mut(crate::core_3d::graph::NAME)
            .unwrap();
        draw_3d_graph.add_node(draw_3d_graph::node::DLSS, dlss_node);
        draw_3d_graph.add_slot_edge(
            draw_3d_graph.input_node().id,
            crate::core_3d::graph::input::VIEW_ENTITY,
            draw_3d_graph::node::DLSS,
            DlssNode::IN_VIEW,
        );
        // MAIN_PASS -> TAA -> START_POST_PROCESSING
        draw_3d_graph.add_node_edge(
            crate::core_3d::graph::node::MAIN_PASS,
            draw_3d_graph::node::DLSS,
        );
        draw_3d_graph.add_node_edge(
            draw_3d_graph::node::DLSS,
            crate::core_3d::graph::node::BLOOM,
        );
        // TODO: START_POST_PROCESSING
        // draw_3d_graph.add_node_edge(
        //     draw_3d_graph::node::DLSS,
        //     crate::core_3d::graph::node::TONEMAPPING,
        // );
    }
}

struct DlssNode {
    view_query: QueryState<(
        &'static ExtractedView,
        &'static DlssSettings,
        &'static ViewportOverride,
        &'static TemporalJitter,
        &'static ViewTarget,
        &'static ViewPrepassTextures,
    )>,
}

impl DlssNode {
    const IN_VIEW: &'static str = "view";

    fn new(world: &mut World) -> Self {
        Self {
            view_query: QueryState::new(world),
        }
    }
}

impl Node for DlssNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.view_query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        #[cfg(feature = "trace")]
        let _dlss_span = info_span!("dlss").entered();

        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let dlss = world.resource::<DlssResource>();
        // let (
        //     Ok((camera, view_target, taa_history_textures, prepass_textures, taa_pipeline_id)),
        //     Some(pipelines),
        //     Some(pipeline_cache),
        // ) = (
        //     self.view_query.get_manual(world, view_entity),
        // ) else {
        //     return Ok(());
        // };
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
        let dlss_context = dlss.get_context(
            upscaled_resolution,
            dlss_settings.preset,
            view.hdr,
            render_context.command_encoder(),
            render_context.render_device,
        );
        let view_target = view_target.post_process_write();

        dlss_context.render(todo!());

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
    fn get_context(
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
                    &mut maybe_command_encoder.unwrap(),
                )
                .expect("Failed to create DlssContext");

                (Mutex::new(dlss_context), true)
            });

        dlss_context.1 = true;
        dlss_context.0.lock().unwrap()
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
            commands
                .get_or_spawn(entity)
                .insert((dlss_settings.clone(), camera_projection.clone()));

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
    )>,
    mut dlss: NonSendMut<DlssResource>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    frame_count: Res<FrameCount>,
    mut commands: Commands,
) {
    let mut maybe_command_encoder = None;

    for (entity, view, dlss_settings, camera, mut temporal_jitter) in &mut query {
        let upscaled_resolution = view.viewport.zw();
        let dlss_context = dlss.get_context(
            upscaled_resolution,
            dlss_settings.preset,
            view.hdr,
            &mut maybe_command_encoder,
            &render_device,
        );
        let render_resolution = dlss_context.max_render_resolution();

        temporal_jitter.offset = dlss_context.suggested_jitter(frame_count.0) + 0.5;

        commands.entity(entity).insert(ViewportOverride(Viewport {
            physical_position: view.viewport.xy(),
            physical_size: render_resolution,
            depth: camera.viewport.map(|v| v.depth).unwap_or(0.0..1.0),
        }));
    }

    if let Some(command_encoder) = maybe_command_encoder {
        render_queue.submit([command_encoder.finish()]);
    }

    dlss.drop_stale_contexts();
}
