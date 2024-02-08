use super::{
    asset_binder::AssetBindings, blas_manager::BlasManager,
    extract_asset_events::ExtractedAssetEvents, gpu_types::GpuSolariMaterial,
};
use crate::StandardMaterial;
use bevy_asset::{AssetId, Handle};
use bevy_ecs::{
    system::{Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_math::Mat4;
use bevy_render::{
    mesh::Mesh,
    render_resource::*,
    renderer::{RenderDevice, RenderQueue},
    texture::Image,
    Extract,
};
use bevy_transform::components::GlobalTransform;
use bevy_utils::HashMap;
use std::iter;

#[derive(Resource, Default)]
pub struct ExtractedScene {
    entities: Vec<(AssetId<Mesh>, AssetId<StandardMaterial>, GlobalTransform)>,
}

pub fn extract_scene(
    mut scene: ResMut<ExtractedScene>,
    query: Extract<Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &GlobalTransform)>>,
) {
    scene.entities.clear();

    for (mesh_handle, material_handle, transform) in &query {
        scene
            .entities
            .push((mesh_handle.id(), material_handle.id(), transform.clone()));
    }
}

#[derive(Resource)]
pub struct SceneBindings {
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: Option<BindGroup>,
}

impl FromWorld for SceneBindings {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        Self {
            bind_group_layout: render_device.create_bind_group_layout(
                "solari_scene_bind_group_layout",
                &BindGroupLayoutEntries::sequential(
                    ShaderStages::COMPUTE,
                    (
                        BindingType::AccelerationStructure,
                        // TODO: AS->mesh/material mapping
                        // TODO: Mesh transforms
                        // TODO: Materials
                        // TODO: Lights
                    ),
                ),
            ),
            bind_group: None,
        }
    }
}

pub fn prepare_scene_bindings(
    mut scene_bindings: ResMut<SceneBindings>,
    asset_bindings: Res<AssetBindings>,
    scene: Res<ExtractedScene>,
    asset_events: Res<ExtractedAssetEvents>,
    blas_manager: Res<BlasManager>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    // Build buffer of materials
    let mut materials = Vec::with_capacity(asset_events.materials.len());
    let mut material_ids = HashMap::with_capacity(asset_events.materials.len());
    for (asset_id, material) in &asset_events.materials {
        let get_image_id = |asset_id: Option<AssetId<Image>>| match asset_id {
            Some(asset_id) => *asset_bindings
                .image_indices
                .get(&asset_id)
                .unwrap_or(&u32::MAX),
            None => u32::MAX,
        };

        material_ids.insert(*asset_id, materials.len() as u32);
        materials.push(GpuSolariMaterial {
            base_color: material.base_color.as_linear_rgba_f32(),
            base_color_texture_id: get_image_id(material.base_color_texture),
            normal_map_texture_id: get_image_id(material.normal_map_texture),
            emissive: material.emissive.as_linear_rgba_f32(),
            emissive_texture_id: get_image_id(material.emissive_texture),
        });
    }

    // Create TLAS
    let mut tlas = TlasPackage::new(
        render_device
            .wgpu_device()
            .create_tlas(&CreateTlasDescriptor {
                label: Some("tlas"),
                flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
                update_mode: AccelerationStructureUpdateMode::Build,
                max_instances: scene.entities.len() as u32,
            }),
        scene.entities.len() as u32,
    );

    // Build each entity into the TLAS and push its transform/mesh_id/material_id to a GPU buffer
    let mut entity_i = 0;
    let mut transforms = Vec::with_capacity(scene.entities.len());
    let mut mesh_material_ids = Vec::with_capacity(scene.entities.len());
    for (mesh_id, material_id, transform) in &scene.entities {
        if let (Some(blas), Some(mesh_id), Some(material_id)) = (
            blas_manager.get(mesh_id),
            asset_bindings.mesh_indices.get(mesh_id),
            material_ids.get(material_id),
        ) {
            let transform = transform.compute_matrix();
            transforms.push(transform);

            // TODO: Check for ID overflow
            mesh_material_ids.push((*mesh_id << 16) | *material_id);

            *tlas.get_mut_single(entity_i).unwrap() = Some(TlasInstance::new(
                blas,
                tlas_transform(&transform),
                entity_i as u32, // TODO: Max 24 bits
                0xFF,
            ));

            entity_i += 1;
        }
    }

    // Build TLAS
    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("build_tlas_command_encoder"),
    });
    command_encoder.build_acceleration_structures(&[], iter::once(&tlas));
    render_queue.submit([command_encoder.finish()]);

    // Create a bind group for the created resources
    scene_bindings.bind_group = Some(render_device.create_bind_group(
        "solari_scene_bind_group",
        &scene_bindings.bind_group_layout,
        &BindGroupEntries::sequential((
            tlas.as_binding(),
            // TODO: Other bindings
        )),
    ));
}

fn tlas_transform(transform: &Mat4) -> [f32; 12] {
    transform.transpose().to_cols_array()[..12]
        .try_into()
        .unwrap()
}
