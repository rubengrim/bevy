pub mod bind_group;
pub mod bind_group_layout;
pub mod blas;
pub mod material;
mod misc;

use self::{
    bind_group::{queue_scene_bind_group, SolariSceneBindGroup},
    bind_group_layout::SolariSceneResources,
    blas::{prepare_blas, BlasStorage},
    material::SolariMaterial,
};
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, AddAsset, Assets, Handle, HandleUntyped};
use bevy_ecs::{
    prelude::Entity,
    schedule::IntoSystemConfigs,
    system::{Commands, Query, Res},
};
use bevy_reflect::TypeUuid;
use bevy_render::{
    prelude::Mesh, render_resource::Shader, Extract, ExtractSchedule, Render, RenderApp, RenderSet,
};
use bevy_transform::prelude::GlobalTransform;

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

        app.add_asset::<SolariMaterial>();

        app.sub_app_mut(RenderApp)
            .init_resource::<BlasStorage>()
            .init_resource::<SolariSceneResources>()
            .init_resource::<SolariSceneBindGroup>()
            .add_systems(ExtractSchedule, extract_scene)
            .add_systems(Render, prepare_blas.in_set(RenderSet::Prepare))
            .add_systems(Render, queue_scene_bind_group.in_set(RenderSet::Queue));
    }
}

fn extract_scene(
    meshes: Extract<
        Query<(
            Entity,
            &Handle<Mesh>,
            &Handle<SolariMaterial>,
            &GlobalTransform,
        )>,
    >,
    materials: Extract<Res<Assets<SolariMaterial>>>,
    mut commands: Commands,
) {
    commands.insert_or_spawn_batch(
        meshes
            .iter()
            .filter_map(|(entity, mesh_handle, material_handle, transform)| {
                materials.get(material_handle).map(|material| {
                    (
                        entity,
                        (
                            mesh_handle.clone_weak(),
                            material_handle.clone_weak(),
                            material.clone(),
                            transform.clone(),
                        ),
                    )
                })
            })
            .collect::<Vec<_>>(),
    );
}
