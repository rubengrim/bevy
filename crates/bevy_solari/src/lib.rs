mod blas;
mod bundle;
mod material;
mod material_buffer;
mod misc;
mod node;
mod pipeline;
mod tlas;

pub use crate::bundle::{SolariCamera3dBundle, SolariMaterialMeshBundle};
pub use crate::material::SolariMaterial;

use crate::{
    blas::{prepare_blas, BlasStorage},
    material_buffer::{prepare_material_buffer, MaterialBuffer},
    misc::extract_meshes,
    node::SolariNode,
    pipeline::{prepare_pipelines, SolariPipeline, SOLARI_SHADER_HANDLE},
    tlas::{prepare_tlas, TlasResource},
};
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, AddAsset, HandleUntyped};
use bevy_core_pipeline::{
    core_3d::graph::node::{TONEMAPPING, UPSCALING},
    tonemapping::TonemappingNode,
    upscaling::UpscalingNode,
};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::Resource;
use bevy_reflect::TypeUuid;
use bevy_render::{
    render_graph::RenderGraphApp,
    render_resource::{Shader, SpecializedComputePipelines},
    renderer::RenderDevice,
    settings::WgpuFeatures,
    ExtractSchedule, Render, RenderApp, RenderSet,
};

const SOLARI_GRAPH: &str = "solari";
const SOLARI_NODE: &str = "solari";

const MATERIAL_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2717171717171755);

// TODO: Document valid mesh attributes and layout

#[derive(Default)]
pub struct SolariPlugin;

impl Plugin for SolariPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SOLARI_SHADER_HANDLE, "solari.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            MATERIAL_SHADER_HANDLE,
            "material.wgsl",
            Shader::from_wgsl
        );

        let needed_features = WgpuFeatures::RAY_TRACING_ACCELERATION_STRUCTURE
            | WgpuFeatures::RAY_QUERY
            // TODO: Needed?
            // | WgpuFeatures::TEXTURE_BINDING_ARRAY
            // | WgpuFeatures::BUFFER_BINDING_ARRAY
            // | WgpuFeatures::STORAGE_RESOURCE_BINDING_ARRAY
            | WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | WgpuFeatures::PARTIALLY_BOUND_BINDING_ARRAY;

        match app.world.get_resource::<RenderDevice>() {
            Some(render_device) if render_device.features().contains(needed_features) => {}
            _ => return,
        }

        app.insert_resource(SolariSupported)
            .add_asset::<SolariMaterial>();

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .add_render_sub_graph(SOLARI_GRAPH)
            .add_render_graph_node::<SolariNode>(SOLARI_GRAPH, SOLARI_NODE)
            .add_render_graph_node::<TonemappingNode>(SOLARI_GRAPH, TONEMAPPING)
            .add_render_graph_node::<UpscalingNode>(SOLARI_GRAPH, UPSCALING)
            .add_render_graph_edges(SOLARI_GRAPH, &[SOLARI_NODE, TONEMAPPING, UPSCALING]);

        render_app
            .init_resource::<SolariPipeline>()
            .init_resource::<SpecializedComputePipelines<SolariPipeline>>()
            .init_resource::<MaterialBuffer>()
            .init_resource::<BlasStorage>()
            .init_resource::<TlasResource>()
            .add_systems(ExtractSchedule, extract_meshes)
            .add_systems(
                Render,
                (prepare_blas, prepare_tlas)
                    .chain()
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                (prepare_pipelines, prepare_material_buffer).in_set(RenderSet::Prepare),
            );
    }
}

#[derive(Resource)]
pub struct SolariSupported;
