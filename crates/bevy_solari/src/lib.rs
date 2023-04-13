mod blas;
mod material;
mod misc;
mod node;
mod pipeline;
mod tlas;

pub use material::{SolariMaterial, SolariMaterialMeshBundle};

use crate::{
    blas::{prepare_blas, BlasStorage},
    material::{prepare_material_buffer, MaterialBuffer},
    misc::{extract_meshes, prepare_textures, queue_view_bind_group},
    node::SolariNode,
    pipeline::{prepare_pipelines, SolariPipeline, SOLARI_SHADER_HANDLE},
    tlas::{prepare_tlas, TlasResource},
};
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_reflect::TypeUuid;
use bevy_render::render_resource::{Shader, SpecializedComputePipelines};
use bevy_render::ExtractSchedule;
use bevy_render::{
    render_graph::RenderGraphApp, renderer::RenderDevice, settings::WgpuFeatures, Render,
    RenderApp, RenderSet,
};

const SOLARI_GRAPH: &str = "solari";
const SOLARI_NODE: &str = "solari";

const MATERIAL_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2717171717171755);

#[derive(Default)]
pub struct SolariPlugin;

impl Plugin for SolariPlugin {
    fn build(&self, app: &mut App) {
        let needed_features =
            WgpuFeatures::RAY_TRACING_ACCELERATION_STRUCTURE | WgpuFeatures::RAY_QUERY;
        match app.world.get_resource::<RenderDevice>() {
            Some(render_device) if render_device.features().contains(needed_features) => {}
            _ => return,
        }

        load_internal_asset!(app, SOLARI_SHADER_HANDLE, "solari.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            MATERIAL_SHADER_HANDLE,
            "material.wgsl",
            Shader::from_wgsl
        );

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .add_render_sub_graph(SOLARI_GRAPH)
            .add_render_graph_node::<SolariNode>(SOLARI_GRAPH, SOLARI_NODE);

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
                (prepare_pipelines, prepare_textures, prepare_material_buffer)
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue_view_bind_group.in_set(RenderSet::Queue));
    }
}
