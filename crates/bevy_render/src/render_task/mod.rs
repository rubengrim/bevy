mod node;
mod prepare_bind_groups;
mod prepare_pipelines;
mod prepare_resources;

pub use self::prepare_resources::{
    RenderTaskResource, RenderTaskResourceRegistry, RenderTaskResourceView,
};

use self::{
    node::RenderTaskNode, prepare_bind_groups::prepare_bind_groups,
    prepare_pipelines::RenderTaskPipelines, prepare_resources::prepare_resources,
};
use crate::{
    render_graph::RenderGraphApp,
    render_resource::{BindGroup, ComputePipeline, Shader},
    Render, RenderSet,
};
use bevy_app::App;
use bevy_asset::Handle;
use bevy_ecs::{component::Component, schedule::IntoSystemConfigs};
use bevy_math::UVec2;
use bevy_utils::HashMap;
use wgpu::CommandEncoder;

// TODO: Write prepare systems
// TODO: Support buffers
// TODO: Figure out how to allow the user to specialize shaders
// TODO: Dedup pipelines / bind group layouts / bind groups
// TODO: Replace hashmaps with compile time hashmaps over strings or marker types
// TODO: Automate generating shader binding wgsl code and loading shaders
// TODO: Replace unwraps with expects
// TODO: Docs

pub trait RenderTask: Send + Sync + 'static {
    type RenderTaskSettings: Component;

    fn name() -> &'static str;

    fn render_node_sub_graph() -> &'static str {
        // bevy_core_pipeline::core_3d::CORE_3D
        "core_3d"
    }

    // TODO: Optional static resource initialization API

    fn render_node_edges() -> &'static [&'static str];

    fn resources(physical_viewport_size: UVec2) -> HashMap<&'static str, RenderTaskResource>;

    fn passes() -> HashMap<&'static str, RenderTaskPass>;

    // TODO: Instead of command encoder, provide tracked compute pass
    fn encode_commands(
        encoder: &mut CommandEncoder,
        pipelines: HashMap<&'static str, &ComputePipeline>,
        bind_groups: HashMap<&'static str, &BindGroup>,
    );
}

pub struct RenderTaskPass {
    pub shader: Handle<Shader>,
    /// Assumed to be the same as the pipeline name if None.
    pub entry_point: Option<&'static str>,
    pub bindings: &'static [RenderTaskResourceView],
}

impl RenderTaskPass {
    pub fn new(shader: Handle<Shader>, resources: &'static [RenderTaskResourceView]) -> Self {
        Self {
            shader,
            entry_point: None,
            bindings: resources,
        }
    }
}

pub(crate) fn add_render_task_to_render_app<R: RenderTask>(render_app: &mut App) {
    render_app
        .insert_resource(RenderTaskPipelines::<R>::new())
        .add_render_graph_node::<RenderTaskNode<R>>(R::render_node_sub_graph(), R::name())
        .add_render_graph_edges(R::render_node_sub_graph(), R::render_node_edges())
        .add_systems(
            Render,
            (
                RenderTaskPipelines::<R>::prepare_pipelines.in_set(RenderSet::PrepareResources),
                (
                    prepare_resources::<R>.in_set(RenderSet::PrepareResources),
                    prepare_bind_groups::<R>.in_set(RenderSet::PrepareBindGroups),
                )
                    .chain(),
            ),
        );
}
