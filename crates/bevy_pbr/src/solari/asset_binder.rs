use super::extract_asset_events::ExtractedAssetEvents;
use bevy_asset::AssetId;
use bevy_ecs::{
    system::{Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::{
    mesh::{GpuBufferInfo, Mesh},
    render_asset::RenderAssets,
    render_resource::{
        binding_types::texture_2d_array, BindGroup, BindGroupEntries, BindGroupLayout,
        BindGroupLayoutEntries, ShaderStages, TextureDimension, TextureSampleType,
    },
    renderer::RenderDevice,
    texture::{FallbackImage, Image},
};
use bevy_utils::HashMap;
use std::ops::Deref;

#[derive(Resource)]
pub struct AssetBindings {
    pub bind_group_layout: BindGroupLayout,
    pub image_indices: HashMap<AssetId<Image>, u32>,
    pub mesh_indices: HashMap<AssetId<Mesh>, u32>,
    pub bind_group: Option<BindGroup>,
}

impl FromWorld for AssetBindings {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        Self {
            bind_group_layout: render_device.create_bind_group_layout(
                "solari_assets_bind_group_layout",
                &BindGroupLayoutEntries::sequential(
                    ShaderStages::COMPUTE,
                    (
                        texture_2d_array(TextureSampleType::Float { filterable: true }),
                        todo!("storage_buffer_array_read_only_sized"),
                        todo!("storage_buffer_array_read_only_sized"),
                    ),
                ),
            ),
            image_indices: HashMap::default(),
            mesh_indices: HashMap::default(),
            bind_group: None,
        }
    }
}

pub fn update_asset_binding_arrays(
    mut asset_bindings: ResMut<AssetBindings>,
    asset_events: Res<ExtractedAssetEvents>,
    render_images: Res<RenderAssets<Image>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    fallback_image: Res<FallbackImage>,
    render_device: Res<RenderDevice>,
) {
    if asset_events.images_changed.is_empty() && asset_events.meshes_changed.is_empty() {
        return;
    }

    asset_bindings.image_indices.clear();
    asset_bindings.mesh_indices.clear();

    let mut images = render_images
        .iter()
        .filter(|(_, image)| {
            todo!("Verify texture binding usage, d2 dimension, sample_count 1, float")
        })
        .enumerate()
        .map(|(i, (asset_id, image))| {
            asset_bindings.image_indices.insert(asset_id, i as u32);
            image.texture_view.deref()
        })
        .collect::<Vec<_>>();
    images.push(&fallback_image.d2.texture_view);

    let (vertex_buffers, index_buffers) = render_meshes
        .iter()
        .filter(|(_, mesh)| todo!("Filter mesh by indexed with u32s"))
        .enumerate()
        .map(|(i, (asset_id, mesh))| {
            asset_bindings.mesh_indices.insert(asset_id, i as u32);
            let index_buffer = match &mesh.buffer_info {
                GpuBufferInfo::Indexed { buffer, .. } => buffer,
                GpuBufferInfo::NonIndexed => unreachable!(),
            };
            (
                mesh.vertex_buffer.as_entire_buffer_binding(),
                index_buffer.as_entire_buffer_binding(),
            )
        })
        .unzip::<_, _, Vec<_>, Vec<_>>();

    if (vertex_buffers.is_empty()) {
        return;
    }

    asset_bindings.bind_group = Some(render_device.create_bind_group(
        "solari_assets_bind_group",
        &asset_bindings.bind_group_layout,
        &BindGroupEntries::sequential((
            images.as_slice(),
            vertex_buffers.as_slice(),
            index_buffers.as_slice(),
        )),
    ));
}
