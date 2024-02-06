use super::extract_asset_events::ExtractedAssetEvents;
use bevy_asset::AssetId;
use bevy_ecs::{
    system::{Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::{
    mesh::Mesh,
    render_asset::RenderAssets,
    render_resource::{BindGroup, BindGroupLayout},
    renderer::RenderDevice,
    texture::Image,
};
use bevy_utils::HashMap;

#[derive(Resource)]
pub struct AssetBindings {
    pub bind_group_layout: BindGroupLayout,
    pub mesh_indices: HashMap<AssetId<Mesh>, u32>,
    pub image_indices: HashMap<AssetId<Image>, u32>,
    pub bind_group: BindGroup,
}

impl FromWorld for AssetBindings {
    fn from_world(world: &mut World) -> Self {
        let render_device = render_world.resource::<RenderDevice>();

        Self {
            bind_group_layout: todo!(),
            mesh_indices: todo!(),
            image_indices: todo!(),
            bind_group: todo!(),
        }
    }
}

pub fn update_asset_binding_arrays(
    asset_bindings: ResMut<AssetBindings>,
    asset_events: Res<ExtractedAssetEvents>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_images: Res<RenderAssets<Image>>,
    render_device: Res<RenderDevice>,
) {
    if asset_events.meshes_changed.is_empty() && asset_events.images_changed.is_empty() {
        return;
    }

    todo!()
}
