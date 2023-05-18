use crate::SolariMaterial;
use bevy_asset::{Assets, Handle};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res},
};
use bevy_math::Mat4;
use bevy_render::{prelude::Mesh, render_resource::ShaderType, Extract};
use bevy_transform::prelude::GlobalTransform;

pub fn extract_scene(
    meshes: Extract<
        Query<(
            Entity,
            &Handle<Mesh>,
            &Handle<SolariMaterial>,
            &GlobalTransform,
            &PreviousGlobalTransform,
        )>,
    >,
    materials: Extract<Res<Assets<SolariMaterial>>>,
    mut commands: Commands,
) {
    commands.insert_or_spawn_batch(
        meshes
            .iter()
            .filter_map(
                |(entity, mesh_handle, material_handle, transform, previous_transform)| {
                    materials.get(material_handle).map(|material| {
                        (
                            entity,
                            (
                                mesh_handle.clone_weak(),
                                material_handle.clone_weak(),
                                material.clone(),
                                transform.clone(),
                                previous_transform.clone(),
                            ),
                        )
                    })
                },
            )
            .collect::<Vec<_>>(),
    );
}

#[derive(Component, ShaderType, Clone)]
pub struct PreviousGlobalTransform {
    pub t: Mat4,
}

pub fn update_mesh_previous_global_transforms(
    mut commands: Commands,
    meshes: Query<(Entity, &GlobalTransform), With<Handle<Mesh>>>,
) {
    for (entity, transform) in &meshes {
        commands.entity(entity).insert(PreviousGlobalTransform {
            t: transform.compute_matrix(),
        });
    }
}
