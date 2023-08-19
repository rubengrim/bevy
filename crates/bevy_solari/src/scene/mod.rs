pub mod bind_group;
pub mod bind_group_layout;
pub mod blas;
pub mod material;
mod misc;
mod scene;
pub mod uniforms;

use self::{
    bind_group::{queue_scene_bind_group, SolariSceneBindGroup},
    bind_group_layout::SolariSceneBindGroupLayout,
    blas::{prepare_blas, BlasStorage},
    material::SolariMaterial,
    scene::{
        ensure_necessary_vertex_attributes, extract_scene, update_mesh_previous_global_transforms,
    },
};
use bevy_app::{App, Plugin, PreUpdate, Update};
use bevy_asset::{load_internal_asset, AddAsset, HandleUntyped};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_reflect::TypeUuid;
use bevy_render::{render_resource::Shader, ExtractSchedule, Render, RenderApp, RenderSet};

// TODO: Document valid mesh attributes + layout + indices

pub struct SolariScenePlugin;

const SOLARI_SCENE_TYPES_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1717171717171755);
const SOLARI_SCENE_BINDINGS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2717171717171755);

impl Plugin for SolariScenePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SOLARI_SCENE_TYPES_SHADER,
            "scene_types.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_SCENE_BINDINGS_SHADER,
            "scene_bindings.wgsl",
            Shader::from_wgsl
        );

        app.add_asset::<SolariMaterial>()
            .add_systems(PreUpdate, update_mesh_previous_global_transforms)
            .add_systems(Update, ensure_necessary_vertex_attributes);

        app.sub_app_mut(RenderApp)
            .init_resource::<BlasStorage>()
            .init_resource::<SolariSceneBindGroupLayout>()
            .init_resource::<SolariSceneBindGroup>()
            .add_systems(ExtractSchedule, extract_scene)
            .add_systems(Render, prepare_blas.in_set(RenderSet::Prepare))
            .add_systems(Render, queue_scene_bind_group.in_set(RenderSet::Queue));
    }
}
