use bevy_asset::{AssetEvent, AssetId};
use bevy_ecs::{
    event::EventReader,
    system::{ResMut, Resource, SystemState},
    world::{FromWorld, Mut, World},
};
use bevy_render::{mesh::Mesh, texture::Image, MainWorld};
use bevy_utils::HashSet;

#[derive(Resource, Default)]
pub struct ExtractedAssetEvents {
    pub meshes_changed: HashSet<AssetId<Mesh>>,
    pub meshes_removed: Vec<AssetId<Mesh>>,
    pub images_changed: HashSet<AssetId<Image>>,
    pub images_removed: Vec<AssetId<Image>>,
}

pub fn extract_asset_events(
    mut main_world: ResMut<MainWorld>,
    mut events: ResMut<ExtractedAssetEvents>,
) {
    events.meshes_changed.clear();
    events.meshes_removed.clear();
    events.images_changed.clear();
    events.images_removed.clear();

    main_world.resource_scope(
        |main_world, mut state: Mut<ExtractAssetEventsSystemState>| {
            let (mut mesh_events, mut image_events) = state.state.get(main_world);

            for asset_event in mesh_events.read() {
                match asset_event {
                    AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                        events.meshes_changed.insert(*id);
                    }
                    AssetEvent::Unused { id } => {
                        events.meshes_removed.push(*id);
                        events.meshes_changed.remove(id);
                    }
                    _ => {}
                }
            }

            for asset_event in image_events.read() {
                match asset_event {
                    AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                        events.images_changed.insert(*id);
                    }
                    AssetEvent::Unused { id } => {
                        events.images_removed.push(*id);
                        events.images_changed.remove(id);
                    }
                    _ => {}
                }
            }
        },
    );
}

#[derive(Resource)]
pub struct ExtractAssetEventsSystemState {
    state: SystemState<(
        EventReader<'static, 'static, AssetEvent<Mesh>>,
        EventReader<'static, 'static, AssetEvent<Image>>,
    )>,
}

impl FromWorld for ExtractAssetEventsSystemState {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: SystemState::new(world),
        }
    }
}
