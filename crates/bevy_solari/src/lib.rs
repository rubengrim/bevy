mod blas;
mod misc;
mod node;
mod pipeline;
mod tlas;

use crate::blas::{prepare_blas, BlasStorage};
use crate::misc::{extract_transforms, queue_view_bind_group};
use crate::node::SolariNode;
use crate::pipeline::SolariPipeline;
use crate::tlas::{prepare_tlas, TlasResource};
use bevy_app::{App, Plugin};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_render::ExtractSchedule;
use bevy_render::{
    render_graph::RenderGraphApp, renderer::RenderDevice, settings::WgpuFeatures, Render,
    RenderApp, RenderSet,
};

const SOLARI_GRAPH: &str = "solari";
const SOLARI_NODE: &str = "solari";

#[derive(Default)]
pub struct SolariPlugin;

impl Plugin for SolariPlugin {
    fn build(&self, app: &mut App) {
        // TODO: On headless, RenderDevice won't exist
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
            .init_resource::<SolariPipeline>()
            .init_resource::<BlasStorage>()
            .init_resource::<TlasResource>()
            .add_systems(ExtractSchedule, extract_transforms)
            .add_systems(
                Render,
                (prepare_blas, prepare_tlas)
                    .chain()
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue_view_bind_group.in_set(RenderSet::Queue));
    }
}
