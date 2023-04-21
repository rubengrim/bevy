use crate::SOLARI_GRAPH;
use bevy_core_pipeline::tonemapping::Tonemapping;
use bevy_ecs::{
    prelude::{Bundle, Component},
    system::Query,
};
use bevy_render::{
    camera::CameraRenderGraph,
    extract_component::ExtractComponent,
    prelude::{Camera, Projection},
    view::ColorGrading,
};
use bevy_transform::prelude::{GlobalTransform, Transform};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

#[derive(Bundle)]
pub struct SolariPathTracerCamera3dBundle {
    pub path_tracer: SolariPathTracer,
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: Projection,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub tonemapping: Tonemapping,
    pub color_grading: ColorGrading,
}

impl Default for SolariPathTracerCamera3dBundle {
    fn default() -> Self {
        Self {
            path_tracer: Default::default(),
            camera_render_graph: CameraRenderGraph::new(SOLARI_GRAPH),
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            projection: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            tonemapping: Tonemapping::TonyMcMapface,
            color_grading: Default::default(),
        }
    }
}

#[derive(Component, ExtractComponent, Clone, Default)]
pub struct SolariPathTracer {
    pub sample_count: Arc<AtomicU32>,
    pub last_transform: GlobalTransform,
}

pub fn reset_accumulation_on_camera_movement(
    mut views: Query<(&mut SolariPathTracer, &GlobalTransform)>,
) {
    for (mut path_tracer, transform) in &mut views {
        if path_tracer.last_transform != *transform {
            path_tracer.sample_count.store(0, Ordering::SeqCst);
            path_tracer.last_transform = *transform;
        }
    }
}
