use crate::SOLARI_GRAPH;
use bevy_core_pipeline::tonemapping::Tonemapping;
use bevy_ecs::{
    prelude::{Bundle, Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_math::Mat4;
use bevy_render::{
    camera::CameraRenderGraph,
    extract_component::ExtractComponent,
    prelude::{Camera, Projection},
    render_resource::{DynamicUniformBuffer, ShaderType},
    renderer::{RenderDevice, RenderQueue},
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
