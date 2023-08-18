use crate::SOLARI_GRAPH;
use bevy_core::FrameCount;
use bevy_core_pipeline::tonemapping::Tonemapping;
use bevy_ecs::{
    prelude::{Bundle, Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_math::{vec2, Mat4};
use bevy_render::{
    camera::{CameraRenderGraph, TemporalJitter},
    extract_component::ExtractComponent,
    prelude::{Camera, Projection},
    render_resource::{DynamicUniformBuffer, ShaderType},
    renderer::{RenderDevice, RenderQueue},
    view::ColorGrading,
};
use bevy_transform::prelude::{GlobalTransform, Transform};

#[derive(Component, ExtractComponent, Clone, Default)]
pub struct SolariSettings {
    pub debug_view: Option<SolariDebugView>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum SolariDebugView {
    Depth,
    WorldNormals,
    MotionVectors,
    BaseColors,
    WorldCacheIrradiance,
    ScreenProbesUnfiltered,
    ScreenProbesFiltered,
    DirectLight,
    IndirectLight,
}

#[derive(Bundle)]
pub struct SolariCamera3dBundle {
    pub solari_settings: SolariSettings,
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: Projection,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub tonemapping: Tonemapping,
    pub color_grading: ColorGrading,
    // TODO: Enable TAA
    // pub taa_jitter: TemporalJitter,
}

impl Default for SolariCamera3dBundle {
    fn default() -> Self {
        Self {
            solari_settings: Default::default(),
            camera_render_graph: CameraRenderGraph::new(SOLARI_GRAPH),
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            projection: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            tonemapping: Default::default(),
            color_grading: Default::default(),
            // taa_jitter: TemporalJitter::default(),
        }
    }
}

#[derive(Component, ExtractComponent, ShaderType, Clone)]
pub struct PreviousViewProjection {
    pub view_proj: Mat4,
}

pub fn update_previous_view_projections(
    mut commands: Commands,
    query: Query<(Entity, &Camera, &GlobalTransform), With<SolariSettings>>,
) {
    for (entity, camera, camera_transform) in &query {
        commands.entity(entity).insert(PreviousViewProjection {
            view_proj: camera.projection_matrix() * camera_transform.compute_matrix().inverse(),
        });
    }
}

#[derive(Resource, Default)]
pub struct PreviousViewProjectionUniforms {
    pub uniforms: DynamicUniformBuffer<PreviousViewProjection>,
}

#[derive(Component)]
pub struct PreviousViewProjectionUniformOffset {
    pub offset: u32,
}

pub fn prepare_previous_view_projection_uniforms(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut view_uniforms: ResMut<PreviousViewProjectionUniforms>,
    views: Query<(Entity, &PreviousViewProjection), With<SolariSettings>>,
) {
    view_uniforms.uniforms.clear();

    for (entity, previous_view_proj) in &views {
        commands
            .entity(entity)
            .insert(PreviousViewProjectionUniformOffset {
                offset: view_uniforms.uniforms.push(previous_view_proj.clone()),
            });
    }

    view_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}

pub fn prepare_taa_jitter(
    mut views: Query<&mut TemporalJitter, With<SolariSettings>>,
    frame_count: Res<FrameCount>,
) {
    // Halton sequence (2, 3) - 0.5, skipping i = 0
    let halton_sequence = [
        vec2(0.0, -0.16666666),
        vec2(-0.25, 0.16666669),
        vec2(0.25, -0.3888889),
        vec2(-0.375, -0.055555552),
        vec2(0.125, 0.2777778),
        vec2(-0.125, -0.2777778),
        vec2(0.375, 0.055555582),
        vec2(-0.4375, 0.3888889),
    ];

    let offset = halton_sequence[frame_count.0 as usize % halton_sequence.len()];

    for mut jitter in &mut views {
        jitter.offset = offset;
    }
}
