mod asset_binder;
mod blas_manager;
mod extract_asset_events;
mod gpu_types;
mod scene_binder;

use self::{
    asset_binder::{prepare_asset_binding_arrays, AssetBindings},
    blas_manager::{prepare_new_blas, BlasManager},
    extract_asset_events::{
        extract_asset_events, ExtractAssetEventsSystemState, ExtractedAssetEvents,
    },
    scene_binder::{extract_scene, prepare_scene_bindings, ExtractedScene, SceneBindings},
};
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, Handle};
use bevy_ecs::{
    schedule::{common_conditions::resource_exists, IntoSystemConfigs},
    system::Resource,
};
use bevy_render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    mesh::Mesh,
    render_asset::prepare_assets,
    render_resource::Shader,
    renderer::RenderDevice,
    settings::WgpuFeatures,
    texture::Image,
    ExtractSchedule, Render, RenderApp, RenderSet,
};

const SOLARI_BINDINGS_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(1717171717171717);

/// TODO: Docs
pub struct SolariPlugin;

impl Plugin for SolariPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SOLARI_BINDINGS_SHADER_HANDLE,
            "solari_bindings.wgsl",
            Shader::from_wgsl
        );
    }

    fn finish(&self, app: &mut App) {
        match app.world.get_resource::<RenderDevice>() {
            Some(render_device) if render_device.features().contains(Self::required_features()) => {
            }
            _ => return,
        }

        app.insert_resource(SolariSupported)
            .add_plugins(ExtractResourcePlugin::<SolariEnabled>::default())
            .init_resource::<ExtractAssetEventsSystemState>();

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .init_resource::<ExtractedAssetEvents>()
            .init_resource::<ExtractedScene>()
            .init_resource::<BlasManager>()
            .init_resource::<AssetBindings>()
            .init_resource::<SceneBindings>()
            .add_systems(
                ExtractSchedule,
                (extract_asset_events, extract_scene).run_if(resource_exists::<SolariEnabled>),
            )
            .add_systems(
                Render,
                (
                    prepare_new_blas
                        .in_set(RenderSet::PrepareAssets)
                        .after(prepare_assets::<Mesh>),
                    prepare_asset_binding_arrays
                        .in_set(RenderSet::PrepareAssets)
                        .after(prepare_assets::<Mesh>)
                        .after(prepare_assets::<Image>),
                    prepare_scene_bindings.in_set(RenderSet::PrepareBindGroups),
                )
                    .run_if(resource_exists::<SolariEnabled>),
            );
    }
}

impl SolariPlugin {
    /// TODO: Docs
    pub fn required_features() -> WgpuFeatures {
        WgpuFeatures::RAY_TRACING_ACCELERATION_STRUCTURE
            | WgpuFeatures::RAY_QUERY
            | WgpuFeatures::TEXTURE_BINDING_ARRAY
            | WgpuFeatures::BUFFER_BINDING_ARRAY
            | WgpuFeatures::STORAGE_RESOURCE_BINDING_ARRAY
            | WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | WgpuFeatures::PARTIALLY_BOUND_BINDING_ARRAY
            | WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | WgpuFeatures::PUSH_CONSTANTS
    }
}

/// TODO: Docs
#[derive(Resource)]
pub struct SolariSupported;

/// TODO: Docs
#[derive(Resource, ExtractResource, Clone)]
pub struct SolariEnabled;
