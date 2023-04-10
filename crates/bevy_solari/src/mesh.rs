use bevy_asset::Handle;
use bevy_ecs::{
    prelude::Entity,
    system::{Commands, Query},
};
use bevy_render::{prelude::Mesh, Extract};

pub fn extract_meshes(mut commands: Commands, meshes: Extract<Query<(Entity, &Handle<Mesh>)>>) {
    let bundles_iter = meshes
        .iter()
        .map(|(entity, mesh)| (entity, mesh.clone_weak()))
        .collect::<Vec<_>>(); // TODO: Avoid Vec
    commands.insert_or_spawn_batch(bundles_iter);
}
