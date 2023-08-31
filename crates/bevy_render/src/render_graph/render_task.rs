use super::{Node, NodeRunError, RenderGraphApp, RenderGraphContext};
use crate::{
    render_resource::{
        BindGroupLayout, ComputePipelineDescriptor, Shader, SpecializedComputePipeline,
    },
    renderer::RenderContext,
    Render, RenderSet,
};
use bevy_app::App;
use bevy_asset::Handle;
use bevy_ecs::{
    schedule::IntoSystemConfigs,
    system::Resource,
    world::{FromWorld, World},
};
use bevy_utils::HashMap;
use std::marker::PhantomData;
use wgpu::CommandEncoder;

pub trait RenderTask: Send + Sync + 'static {
    fn render_node_sub_graph() -> &'static str {
        // bevy_core_pipeline::core_3d::CORE_3D
        "core_3d"
    }

    fn render_node_label() -> &'static str;

    fn render_node_edges() -> &'static [&'static str];

    // TODO: Replace with compile time hashmap
    fn pipelines() -> HashMap<&'static str, RenderTaskPipeline>;

    fn encode_commands(encoder: &mut CommandEncoder);
}

pub enum RenderTaskResource {
    // TODO
}

pub struct RenderTaskPipeline {
    pub shader: Handle<Shader>,
    /// Assumed to be the same as the pipeline name if None.
    pub entry_point: Option<&'static str>,
    pub resources: &'static [RenderTaskResource],
}

impl RenderTaskPipeline {
    pub fn new(shader: Handle<Shader>, resources: &'static [RenderTaskResource]) -> Self {
        Self {
            shader,
            entry_point: None,
            resources,
        }
    }
}

// ----------------------------------------------------------------------------

pub(crate) fn add_render_task_to_render_app<R: RenderTask>(render_app: &mut App) {
    render_app
        .insert_resource(RenderTaskResources::<R>::new())
        .add_render_graph_node::<RenderTaskNode<R>>(
            R::render_node_sub_graph(),
            R::render_node_label(),
        )
        .add_render_graph_edges(R::render_node_sub_graph(), R::render_node_edges())
        .add_systems(
            Render,
            (
                RenderTaskResources::<R>::prepare_pipelines,
                (
                    RenderTaskResources::<R>::prepare_resources,
                    RenderTaskResources::<R>::prepare_bind_groups,
                )
                    .chain(),
            )
                .in_set(RenderSet::PrepareResources),
        );
}

#[derive(Resource)]
struct RenderTaskResources<R: RenderTask> {
    // TODO: Replace with compile time hashmap
    bind_group_layouts: HashMap<&'static str, BindGroupLayout>,
    _marker: PhantomData<R>,
}

impl<R: RenderTask> RenderTaskResources<R> {
    fn new() -> Self {
        Self {
            bind_group_layouts: todo!(),
            _marker: PhantomData,
        }
    }

    fn prepare_pipelines() {
        // TODO
    }

    fn prepare_resources() {
        // TODO
    }

    fn prepare_bind_groups() {
        // TODO
    }
}

impl<R: RenderTask> SpecializedComputePipeline for RenderTaskResources<R> {
    type Key = &'static str;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        let render_task_pipeline = &R::pipelines()[key];

        ComputePipelineDescriptor {
            label: Some(key.into()),
            layout: vec![self.bind_group_layouts[key].clone()],
            push_constant_ranges: vec![],
            shader: render_task_pipeline.shader.clone(),
            shader_defs: vec![], // TODO: Allow specalizing shaders
            entry_point: render_task_pipeline.entry_point.unwrap_or(key).into(),
        }
    }
}

struct RenderTaskNode<R: RenderTask> {
    _marker: PhantomData<R>,
}

impl<R: RenderTask> Node for RenderTaskNode<R> {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        R::encode_commands(render_context.command_encoder());
        Ok(())
    }
}

impl<R: RenderTask> FromWorld for RenderTaskNode<R> {
    fn from_world(_world: &mut World) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
