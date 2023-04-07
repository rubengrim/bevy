mod blas;
mod mesh;
mod node;

use crate::node::SolariNode;
use bevy_app::{App, Plugin};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_render::{
    render_graph::RenderGraphApp, renderer::RenderDevice, settings::WgpuFeatures, ExtractSchedule,
    Render, RenderApp, RenderSet,
};

const SOLARI_GRAPH: &str = "solari";
const SOLARI_NODE: &str = "solari";

#[derive(Default)]
pub struct SolariPlugin;

impl Plugin for SolariPlugin {
    fn build(&self, app: &mut App) {
        let wgpu_features = app.world.resource::<RenderDevice>().features();
        if !wgpu_features
            .contains(WgpuFeatures::RAY_TRACING_ACCELERATION_STRUCTURE | WgpuFeatures::RAY_QUERY)
        {
            return;
        }

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else { return };

        render_app
            .add_render_sub_graph(SOLARI_GRAPH)
            .add_render_graph_node::<SolariNode>(SOLARI_GRAPH, SOLARI_NODE);

        render_app
            .add_systems(ExtractSchedule, mesh::extract_meshes)
            .add_systems(
                Render,
                (mesh::prepare_mesh_transforms, blas::prepare_blas)
                    .chain()
                    .in_set(RenderSet::Prepare),
            );
    }
}
