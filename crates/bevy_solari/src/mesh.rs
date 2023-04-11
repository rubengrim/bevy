use bevy_ecs::{
    prelude::Entity,
    system::{Commands, Query},
};
use bevy_render::Extract;
use bevy_transform::prelude::GlobalTransform;

pub fn extract_transforms(
    mut commands: Commands,
    meshes: Extract<Query<(Entity, &GlobalTransform)>>,
) {
    commands.insert_or_spawn_batch(
        meshes
            .iter()
            .map(|(entity, transform)| (entity, transform.clone()))
            .collect::<Vec<_>>(),
    );
}
