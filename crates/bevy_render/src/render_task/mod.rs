mod node;
mod prepare_bind_groups;
mod prepare_pipelines;
mod prepare_resources;

pub use self::prepare_resources::{RenderTaskResource, RenderTaskTexture};
use self::{
    node::RenderTaskNode, prepare_bind_groups::prepare_bind_groups,
    prepare_pipelines::RenderTaskPipelinesResource, prepare_resources::prepare_resources,
};
use crate::{
    render_graph::RenderGraphApp,
    render_resource::{ComputePipeline, Shader},
    Render, RenderSet,
};
use bevy_app::App;
use bevy_asset::Handle;
use bevy_ecs::{component::Component, schedule::IntoSystemConfigs};
use bevy_utils::HashMap;
use wgpu::CommandEncoder;

// TODO: Replace hashmaps with compile time hashmaps over strings or marker types
// TODO: Docs

pub trait RenderTask: Send + Sync + 'static {
    type RenderTaskSettings: Component;

    fn render_node_sub_graph() -> &'static str {
        // bevy_core_pipeline::core_3d::CORE_3D
        "core_3d"
    }

    fn render_node_label() -> &'static str;

    fn render_node_edges() -> &'static [&'static str];

    fn pipelines() -> HashMap<&'static str, RenderTaskPipelines>;

    fn encode_commands(
        encoder: &mut CommandEncoder,
        pipelines: HashMap<&'static str, &ComputePipeline>,
    );
}

pub struct RenderTaskPipelines {
    pub shader: Handle<Shader>,
    /// Assumed to be the same as the pipeline name if None.
    pub entry_point: Option<&'static str>,
    pub resources: &'static [RenderTaskResource],
}

impl RenderTaskPipelines {
    pub fn new(shader: Handle<Shader>, resources: &'static [RenderTaskResource]) -> Self {
        Self {
            shader,
            entry_point: None,
            resources,
        }
    }
}

pub(crate) fn add_render_task_to_render_app<R: RenderTask>(render_app: &mut App) {
    render_app
        .insert_resource(RenderTaskPipelinesResource::<R>::new())
        .add_render_graph_node::<RenderTaskNode<R>>(
            R::render_node_sub_graph(),
            R::render_node_label(),
        )
        .add_render_graph_edges(R::render_node_sub_graph(), R::render_node_edges())
        .add_systems(
            Render,
            (
                RenderTaskPipelinesResource::<R>::prepare_pipelines
                    .in_set(RenderSet::PrepareResources),
                (
                    prepare_resources::<R>.in_set(RenderSet::PrepareResources),
                    prepare_bind_groups::<R>.in_set(RenderSet::PrepareBindGroups),
                )
                    .chain(),
            ),
        );
}
