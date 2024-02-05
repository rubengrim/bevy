// TODO: Move to bevy_render
mod blas_manager;

use bevy_app::{App, Plugin};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_render::{
    mesh::Mesh, render_asset::prepare_assets, renderer::RenderDevice, settings::WgpuFeatures,
    ExtractSchedule, Render, RenderApp, RenderSet,
};

pub struct SolariPlugin {}

impl Plugin for SolariPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let required_features = WgpuFeatures::RAY_TRACING_ACCELERATION_STRUCTURE
            | WgpuFeatures::RAY_QUERY
            | WgpuFeatures::TEXTURE_BINDING_ARRAY
            | WgpuFeatures::BUFFER_BINDING_ARRAY
            | WgpuFeatures::STORAGE_RESOURCE_BINDING_ARRAY
            | WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | WgpuFeatures::PARTIALLY_BOUND_BINDING_ARRAY
            | WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | WgpuFeatures::PUSH_CONSTANTS;
        match app.world.get_resource::<RenderDevice>() {
            Some(render_device) if render_device.features().contains(required_features) => {}
            _ => return,
        }

        app.init_resource::<blas_manager::ExtractMeshAssetEventsSystemState>();

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<blas_manager::BlasManager>()
            .add_systems(ExtractSchedule, blas_manager::extract_mesh_asset_events)
            .add_systems(
                Render,
                blas_manager::prepare_new_blas
                    .in_set(RenderSet::PrepareAssets)
                    .after(prepare_assets::<Mesh>),
            );
    }
}
