use crate::{SolariMaterial, SolariSun};
use bevy_asset::{AssetEvent, Assets, Handle};
use bevy_ecs::{
    prelude::{Component, Entity, EventReader},
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use bevy_math::Mat4;
use bevy_render::{
    prelude::Mesh,
    render_resource::{PrimitiveTopology, ShaderType},
    Extract,
};
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
    suns: Extract<Query<(Entity, &SolariSun, &GlobalTransform)>>,
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

    commands.insert_or_spawn_batch(
        suns.iter()
            .map(|(entity, sun, transform)| (entity, (sun.clone(), transform.clone())))
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

pub fn ensure_necessary_vertex_attributes(
    mut mesh_events: EventReader<AssetEvent<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // TODO: Parallelize loop
    for event in mesh_events.iter() {
        let handle = match event {
            AssetEvent::Created { handle } => handle,
            _ => continue,
        };

        if let Some(mesh) = meshes.get_mut(handle) {
            if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
                continue;
            }

            if !mesh.contains_attribute(Mesh::ATTRIBUTE_TANGENT) {
                let _ = mesh.generate_tangents();
            }

            if !mesh.contains_attribute(Mesh::ATTRIBUTE_UV_0) {
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_UV_0,
                    // TODO: Avoid this allocation
                    vec![[0.0, 0.0]],
                );
            }
            if !mesh.contains_attribute(Mesh::ATTRIBUTE_TANGENT) {
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_TANGENT,
                    // TODO: Avoid this allocation
                    vec![[0.0, 0.0, 0.0, 0.0]],
                );
            }
        }
    }
}
