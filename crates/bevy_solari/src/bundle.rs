use crate::{material::SolariMaterial, SOLARI_GRAPH};
use bevy_asset::Handle;
use bevy_core_pipeline::tonemapping::Tonemapping;
use bevy_ecs::prelude::Bundle;
use bevy_render::{
    camera::CameraRenderGraph,
    prelude::{Camera, Mesh, Projection},
    view::ColorGrading,
};
use bevy_transform::prelude::{GlobalTransform, Transform};

#[derive(Bundle)]
pub struct SolariCamera3dBundle {
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

#[derive(Bundle, Clone, Default)]
pub struct SolariMaterialMeshBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<SolariMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
