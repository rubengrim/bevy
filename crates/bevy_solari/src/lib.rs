mod path_tracer;
mod scene;
mod solari;

pub use crate::{
    path_tracer::camera::{SolariPathTracer, SolariPathTracerCamera3dBundle},
    scene::material::{SolariMaterial, SolariMaterialMeshBundle},
    solari::camera::{SolariCamera3dBundle, SolariSettings},
};

use crate::{
    path_tracer::{node::SolariPathTracerNode, SolariPathTracerPlugin},
    scene::SolariScenePlugin,
    solari::{node::SolariNode, world_cache::node::SolariWorldCacheNode, SolariRealtimePlugin},
};
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_core_pipeline::{
    core_3d::graph::node::{TONEMAPPING, UPSCALING},
    tonemapping::TonemappingNode,
    upscaling::UpscalingNode,
};
use bevy_ecs::{system::Resource, world::FromWorld};
use bevy_reflect::TypeUuid;
use bevy_render::{
    main_graph::node::CAMERA_DRIVER,
    render_graph::{RenderGraph, RenderGraphApp},
    render_resource::Shader,
    renderer::RenderDevice,
    settings::WgpuFeatures,
    RenderApp,
};

#[derive(Resource)]
pub struct SolariSupported;

#[derive(Default)]
pub struct SolariPlugin;

const SOLARI_GRAPH: &str = "solari_graph";
const SOLARI_NODE: &str = "solari_node";
const SOLARI_WORLD_CACHE_NODE: &str = "solari_world_cache";
const SOLARI_PATH_TRACER_NODE: &str = "solari_path_tracer_node";

const SOLARI_UTILS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4717171717171755);

impl Plugin for SolariPlugin {
    fn build(&self, app: &mut App) {
        let required_features = WgpuFeatures::RAY_TRACING_ACCELERATION_STRUCTURE
            | WgpuFeatures::RAY_QUERY
            | WgpuFeatures::TEXTURE_BINDING_ARRAY
            | WgpuFeatures::BUFFER_BINDING_ARRAY
            | WgpuFeatures::STORAGE_RESOURCE_BINDING_ARRAY
            | WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | WgpuFeatures::PARTIALLY_BOUND_BINDING_ARRAY
            | WgpuFeatures::PUSH_CONSTANTS
            | WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;

        match app.world.get_resource::<RenderDevice>() {
            Some(render_device) if render_device.features().contains(required_features) => {}
            _ => return,
        }

        load_internal_asset!(app, SOLARI_UTILS_SHADER, "utils.wgsl", Shader::from_wgsl);

        app.insert_resource(SolariSupported)
            .add_plugin(SolariScenePlugin)
            .add_plugin(SolariRealtimePlugin)
            .add_plugin(SolariPathTracerPlugin);

        let render_app = &mut app.sub_app_mut(RenderApp);

        let world_cache_node = SolariWorldCacheNode::from_world(&mut render_app.world);
        let render_graph = &mut render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(SOLARI_WORLD_CACHE_NODE, world_cache_node);
        render_graph.add_node_edge(SOLARI_WORLD_CACHE_NODE, CAMERA_DRIVER);

        render_app
            .add_render_sub_graph(SOLARI_GRAPH)
            .add_render_graph_node::<SolariNode>(SOLARI_GRAPH, SOLARI_NODE)
            .add_render_graph_node::<SolariPathTracerNode>(SOLARI_GRAPH, SOLARI_PATH_TRACER_NODE)
            .add_render_graph_node::<TonemappingNode>(SOLARI_GRAPH, TONEMAPPING)
            .add_render_graph_node::<UpscalingNode>(SOLARI_GRAPH, UPSCALING)
            .add_render_graph_edges(
                SOLARI_GRAPH,
                &[SOLARI_NODE, SOLARI_PATH_TRACER_NODE, TONEMAPPING, UPSCALING],
            );
    }
}
