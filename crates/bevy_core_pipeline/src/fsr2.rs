use crate::{
    core_3d::{prepare_core_3d_textures, MainPass3dTexture},
    prelude::Camera3d,
    prepass::{DepthPrepass, VelocityPrepass, ViewPrepassTextures},
};
use bevy_app::{App, Plugin};
use bevy_core::FrameCount;
use bevy_ecs::{
    prelude::{Bundle, Component, Entity},
    query::{QueryState, With},
    schedule::IntoSystemConfig,
    system::{Commands, Query, Res, ResMut, Resource},
    world::World,
};
use bevy_math::{UVec2, Vec4Swizzles};
use bevy_render::{
    camera::TemporalJitter,
    prelude::Projection,
    render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType},
    renderer::{RenderAdapter, RenderContext, RenderDevice},
    texture::CachedTexture,
    view::{ExtractedView, Msaa, ViewTarget},
    Extract, ExtractSchedule, RenderApp, RenderSet,
};
use bevy_time::Time;
#[cfg(feature = "trace")]
use bevy_utils::tracing::info_span;
use bevy_utils::HashMap;
use fsr2_wgpu::{
    Fsr2Context, Fsr2Exposure, Fsr2InitializationFlags, Fsr2ReactiveMask, Fsr2RenderParameters,
    Fsr2Texture,
};
use std::{mem, sync::Mutex};

pub use fsr2_wgpu::Fsr2QualityMode;

mod draw_3d_graph {
    pub mod node {
        /// Label for the FSR2 render node.
        pub const FSR2: &str = "fsr2";
    }
}

pub struct Fsr2Plugin;

impl Plugin for Fsr2Plugin {
    fn build(&self, app: &mut App) {
        if app.get_sub_app_mut(RenderApp).is_err() {
            return;
        }

        app.insert_resource(Msaa::Off);

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        let fsr2_node = Fsr2Node::new(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let draw_3d_graph = graph
            .get_sub_graph_mut(crate::core_3d::graph::NAME)
            .unwrap();
        draw_3d_graph.add_node(draw_3d_graph::node::FSR2, fsr2_node);
        draw_3d_graph.add_slot_edge(
            draw_3d_graph.input_node().id,
            crate::core_3d::graph::input::VIEW_ENTITY,
            draw_3d_graph::node::FSR2,
            Fsr2Node::IN_VIEW,
        );
        // MAIN_PASS -> FSR2 -> BLOOM / TONEMAPPING
        draw_3d_graph.add_node_edge(
            crate::core_3d::graph::node::MAIN_PASS,
            draw_3d_graph::node::FSR2,
        );
        draw_3d_graph.add_node_edge(
            draw_3d_graph::node::FSR2,
            crate::core_3d::graph::node::BLOOM,
        );
        draw_3d_graph.add_node_edge(
            draw_3d_graph::node::FSR2,
            crate::core_3d::graph::node::TONEMAPPING,
        );

        render_app
            .init_resource::<Fsr2ContextCache>()
            .add_system_to_schedule(ExtractSchedule, extract_fsr2_settings)
            .add_system(
                prepare_fsr2
                    .in_set(RenderSet::Prepare)
                    .before(prepare_core_3d_textures),
            );
    }
}

#[derive(Resource, Default)]
pub struct Fsr2ContextCache {
    cache: HashMap<UVec2, (Mutex<Fsr2Context<RenderDevice>>, bool)>,
}

#[derive(Bundle, Default)]
pub struct Fsr2Bundle {
    pub settings: Fsr2Settings,
    pub jitter: TemporalJitter,
    pub depth_prepass: DepthPrepass,
    pub velocity_prepass: VelocityPrepass,
}

#[derive(Component, Clone)]
pub struct Fsr2Settings {
    pub quality_mode: Fsr2QualityMode,
    pub sharpness: f32,
    pub reset: bool,
}

impl Default for Fsr2Settings {
    fn default() -> Self {
        Self {
            quality_mode: Fsr2QualityMode::Performance,
            sharpness: 0.8,
            reset: false,
        }
    }
}

fn extract_fsr2_settings(
    mut commands: Commands,
    query: Extract<
        Query<
            (Entity, &Projection, &Fsr2Settings),
            (
                With<Camera3d>,
                With<TemporalJitter>,
                With<DepthPrepass>,
                With<VelocityPrepass>,
            ),
        >,
    >,
) {
    for (entity, camera_projection, fsr2_settings) in &query {
        if matches!(camera_projection, Projection::Perspective(_)) {
            commands
                .get_or_spawn(entity)
                .insert((fsr2_settings.clone(), camera_projection.clone()));
        }
    }
}

pub fn prepare_fsr2(
    mut fsr2_context_cache: ResMut<Fsr2ContextCache>,
    frame_count: Res<FrameCount>,
    render_device: Res<RenderDevice>,
    mut query: Query<(
        &mut Camera3d,
        &ExtractedView,
        &mut TemporalJitter,
        &Fsr2Settings,
    )>,
) {
    for (mut camera, view, mut temporal_jitter, fsr2_settings) in &mut query {
        let upscaled_resolution = view.viewport.zw();

        let mut fsr2_context = fsr2_context_cache
            .cache
            .entry(upscaled_resolution)
            .or_insert_with(|| {
                let mut initialization_flags = Fsr2InitializationFlags::AUTO_EXPOSURE
                    | Fsr2InitializationFlags::INFINITE_DEPTH
                    | Fsr2InitializationFlags::INVERTED_DEPTH;
                if view.hdr {
                    initialization_flags |= Fsr2InitializationFlags::HIGH_DYNAMIC_RANGE;
                }

                let fsr2_context = Fsr2Context::new(
                    render_device.clone(),
                    upscaled_resolution,
                    upscaled_resolution,
                    initialization_flags,
                )
                .expect("Failed to create Fsr2Context");

                (Mutex::new(fsr2_context), true)
            });
        fsr2_context.1 = true;
        let fsr2_context = fsr2_context.0.lock().unwrap();

        let input_resolution = fsr2_context.suggested_input_resolution(fsr2_settings.quality_mode);
        fsr2_context.suggested_input_resolution(fsr2_settings.quality_mode);

        camera.render_resolution = Some(input_resolution);

        let frame_index = (frame_count.0 % i32::MAX as u32) as i32;
        temporal_jitter.offset =
            fsr2_context.suggested_camera_jitter_offset(input_resolution, frame_index);
        fsr2_context.suggested_camera_jitter_offset(input_resolution, frame_index);
    }

    fsr2_context_cache
        .cache
        .retain(|_, (_, in_use)| mem::take(in_use));
}

struct Fsr2Node {
    view_query: QueryState<(
        &'static Fsr2Settings,
        &'static Camera3d,
        &'static ExtractedView,
        &'static Projection,
        &'static TemporalJitter,
        &'static ViewTarget,
        &'static MainPass3dTexture,
        &'static ViewPrepassTextures,
    )>,
}

impl Fsr2Node {
    const IN_VIEW: &'static str = "view";

    fn new(world: &mut World) -> Self {
        Self {
            view_query: QueryState::new(world),
        }
    }
}

impl Node for Fsr2Node {
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
        let _fsr2_span = info_span!("fsr2").entered();

        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let time = world.resource::<Time>();
        let render_adapter = world.resource::<RenderAdapter>();
        let fsr2_context_cache = world.resource::<Fsr2ContextCache>();
        let Ok((
            fsr2_settings,
            camera_3d,
            view,
            Projection::Perspective(camera_projection),
            temporal_jitter,
            view_target,
            main_pass_3d_texture,
            prepass_textures
        )) = self.view_query.get_manual(world, view_entity) else { return Ok(()) };
        let render_resolution = camera_3d.render_resolution.unwrap();
        let mut fsr2_context = fsr2_context_cache
            .cache
            .get(&view.viewport.zw())
            .unwrap()
            .0
            .lock()
            .unwrap();

        fsr2_context
            .render(Fsr2RenderParameters {
                color: fsr2_texture(&main_pass_3d_texture.texture),
                depth: fsr2_texture(prepass_textures.depth.as_ref().unwrap()),
                motion_vectors: fsr2_texture(prepass_textures.velocity.as_ref().unwrap()),
                motion_vector_scale: Some(render_resolution.as_vec2()),
                exposure: Fsr2Exposure::AutoExposure,
                reactive_mask: Fsr2ReactiveMask::NoMask, // TODO: Auto
                transparency_and_composition_mask: None,
                output: fsr2_texture(view_target.main_texture()),
                input_resolution: render_resolution,
                sharpness: fsr2_settings.sharpness,
                frame_delta_time: time.delta(),
                reset: fsr2_settings.reset,
                camera_near: camera_projection.near,
                camera_far: None,
                camera_fov_angle_vertical: camera_projection.fov,
                jitter_offset: temporal_jitter.offset,
                adapter: render_adapter,
                command_encoder: render_context.command_encoder(),
            })
            .expect("Failed to render FSR2");

        Ok(())
    }
}

fn fsr2_texture(texture: &CachedTexture) -> Fsr2Texture {
    Fsr2Texture {
        texture: &texture.texture,
        view: &texture.default_view,
    }
}
