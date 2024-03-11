mod asset_binder;
mod blas_manager;
mod extract_assets;
mod fallback_blas_builder;
mod fallback_tlas_builder;
mod gpu_types;
mod path_tracer;
mod scene_binder;
mod solari;

use self::{
    asset_binder::{prepare_asset_binding_arrays, AssetBindings},
    blas_manager::{prepare_new_blas, BlasManager},
    extract_assets::{
        extract_asset_events, extract_changed_meshes, ExtractAssetEventsSystemState,
        ExtractedAssetEvents, ExtractedChangedMeshes,
    },
    graph::LabelsSolari,
    path_tracer::{prepare_path_tracer_accumulation_texture, PathTracerNode},
    scene_binder::{extract_scene, prepare_scene_bindings, ExtractedScene, SceneBindings},
    solari::{prepare_view_resources, SolariNode},
};
use crate::DefaultOpaqueRendererMethod;
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, Handle};
use bevy_core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy_ecs::{
    component::Component,
    schedule::{common_conditions::any_with_component, IntoSystemConfigs},
    system::Resource,
};
use bevy_render::{
    extract_component::{ExtractComponent, ExtractComponentPlugin},
    extract_resource::ExtractResource,
    mesh::Mesh,
    render_asset::prepare_assets,
    render_graph::{RenderGraphApp, ViewNodeRunner},
    render_resource::Shader,
    renderer::RenderDevice,
    settings::WgpuFeatures,
    texture::Image,
    view::Msaa,
    ExtractSchedule, Render, RenderApp, RenderSet,
};

pub mod graph {
    use bevy_render::render_graph::RenderLabel;

    #[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
    pub enum LabelsSolari {
        PathTracer,
        Solari,
    }
}

const SOLARI_BINDINGS_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(1717171717171717);
const SOLARI_PATH_TRACER_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(2717171717171717);
const SOLARI_SAMPLE_DIRECT_DIFFUSE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(3717171717171717);

/// TODO: Docs
pub struct SolariPlugin {
    // This is overridden if the supplied type isn't supported by the hardware.
    pub preferred_ray_acceleration_backend_type: SolariRayAccelerationBackendType,
}

impl Default for SolariPlugin {
    fn default() -> Self {
        Self {
            preferred_ray_acceleration_backend_type: SolariRayAccelerationBackendType::Hardware,
        }
    }
}

impl Plugin for SolariPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::Off)
            .insert_resource(DefaultOpaqueRendererMethod::deferred());

        load_internal_asset!(
            app,
            SOLARI_BINDINGS_SHADER_HANDLE,
            "solari_bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_PATH_TRACER_SHADER_HANDLE,
            "path_tracer.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_SAMPLE_DIRECT_DIFFUSE_SHADER_HANDLE,
            "sample_direct_diffuse.wgsl",
            Shader::from_wgsl
        );
    }

    fn finish(&self, app: &mut App) {
        let render_device = app.world.get_resource::<RenderDevice>().unwrap();
        let has_minimal_features = render_device
            .features()
            .contains(Self::required_minimal_features());
        let has_hardware_acceleration_features = render_device
            .features()
            .contains(Self::required_hardware_acceleration_features());

        let backend_type = match self.preferred_ray_acceleration_backend_type {
            SolariRayAccelerationBackendType::Software if has_minimal_features => {
                SolariRayAccelerationBackendType::Software
            }
            SolariRayAccelerationBackendType::Hardware
                if has_minimal_features && has_hardware_acceleration_features =>
            {
                SolariRayAccelerationBackendType::Hardware
            }
            SolariRayAccelerationBackendType::Hardware
                if has_minimal_features && !has_hardware_acceleration_features =>
            {
                SolariRayAccelerationBackendType::Software
            }
            _ => return,
        };

        app.insert_resource(SolariSupported)
            .init_resource::<ExtractAssetEventsSystemState>()
            .add_plugins(ExtractComponentPlugin::<SolariSettings>::default());

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .insert_resource(backend_type.clone())
            .init_resource::<ExtractedAssetEvents>()
            .init_resource::<ExtractedChangedMeshes>()
            .init_resource::<ExtractedScene>()
            .init_resource::<BlasManager>()
            .init_resource::<AssetBindings>()
            .init_resource::<SceneBindings>()
            .add_systems(
                ExtractSchedule,
                (extract_asset_events, extract_scene), //.run_if(any_with_component::<SolariSettings>), // TODO: any_with_component is checking the render world here
            )
            .add_systems(
                Render,
                (
                    prepare_asset_binding_arrays
                        .in_set(RenderSet::PrepareAssets)
                        .after(prepare_assets::<Mesh>)
                        .after(prepare_assets::<Image>),
                    prepare_new_blas
                        .in_set(RenderSet::PrepareAssets)
                        .after(prepare_assets::<Mesh>),
                    prepare_path_tracer_accumulation_texture.in_set(RenderSet::PrepareResources),
                    prepare_view_resources.in_set(RenderSet::PrepareResources),
                    prepare_scene_bindings.in_set(RenderSet::PrepareBindGroups),
                )
                    .run_if(any_with_component::<SolariSettings>),
            )
            .add_render_graph_node::<ViewNodeRunner<PathTracerNode>>(
                Core3d,
                LabelsSolari::PathTracer,
            )
            .add_render_graph_node::<ViewNodeRunner<SolariNode>>(Core3d, LabelsSolari::Solari)
            // .add_render_graph_edges(Core3d, (LabelsSolari::PathTracer, Node3d::EndMainPass))
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::EndMainPass,
                    LabelsSolari::PathTracer,
                    Node3d::Tonemapping,
                ),
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::DeferredPrepass,
                    LabelsSolari::Solari,
                    Node3d::EndMainPass,
                ),
            );

        if backend_type == SolariRayAccelerationBackendType::Software {
            render_app.add_systems(
                ExtractSchedule,
                extract_changed_meshes.after(extract_asset_events),
            );
        }
    }
}

impl SolariPlugin {
    pub fn required_minimal_features() -> WgpuFeatures {
        WgpuFeatures::TEXTURE_BINDING_ARRAY
            | WgpuFeatures::BUFFER_BINDING_ARRAY
            | WgpuFeatures::STORAGE_RESOURCE_BINDING_ARRAY
            | WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | WgpuFeatures::PARTIALLY_BOUND_BINDING_ARRAY
            | WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | WgpuFeatures::PUSH_CONSTANTS
    }

    pub fn required_hardware_acceleration_features() -> WgpuFeatures {
        WgpuFeatures::RAY_TRACING_ACCELERATION_STRUCTURE | WgpuFeatures::RAY_QUERY
    }
}

#[derive(Resource, ExtractResource, Clone, PartialEq, Eq)]
pub enum SolariRayAccelerationBackendType {
    Hardware,
    Software,
}

/// TODO: Docs
#[derive(Resource)]
pub struct SolariSupported;

/// TODO: Docs
// Requires MSAA off, HDR, CameraMainTextureUsages::with_storage_binding(), deferred + depth + motion vector prepass,
//   DefaultOpaqueRendererMethod::deferred, and should disable shadows for all lights
#[derive(Component, ExtractComponent, Clone)]
pub struct SolariSettings {
    pub debug_path_tracer: bool,
}

impl Default for SolariSettings {
    fn default() -> Self {
        Self {
            debug_path_tracer: false,
        }
    }
}
