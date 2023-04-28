use crate::SOLARI_GRAPH;
use bevy_core_pipeline::tonemapping::Tonemapping;
use bevy_ecs::prelude::{Bundle, Component};
use bevy_render::{
    camera::CameraRenderGraph,
    extract_component::ExtractComponent,
    prelude::{Camera, Projection},
    view::ColorGrading,
};
use bevy_transform::prelude::{GlobalTransform, Transform};

#[derive(Component, ExtractComponent, Clone, Default)]
pub struct SolariSettings {}

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
            tonemapping: Tonemapping::TonyMcMapface,
            color_grading: Default::default(),
        }
    }
}
