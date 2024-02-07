use crate::StandardMaterial;
use bevy_asset::{AssetId, Handle};
use bevy_ecs::{
    entity::Entity,
    system::{Query, ResMut, Resource},
};
use bevy_render::{mesh::Mesh, Extract};
use bevy_transform::components::GlobalTransform;

#[derive(Resource, Default)]
pub struct ExtractedScene {
    pub entities: Vec<(
        Entity,
        AssetId<Mesh>,
        AssetId<StandardMaterial>,
        GlobalTransform,
    )>,
}

pub fn extract_scene(
    mut scene: ResMut<ExtractedScene>,
    query: Extract<
        Query<(
            Entity,
            &Handle<Mesh>,
            &Handle<StandardMaterial>,
            &GlobalTransform,
        )>,
    >,
) {
    scene.entities.clear();

    for (entity, mesh_handle, material_handle, transform) in &query {
        scene.entities.push((
            entity,
            mesh_handle.id(),
            material_handle.id(),
            transform.clone(),
        ));
    }
}
