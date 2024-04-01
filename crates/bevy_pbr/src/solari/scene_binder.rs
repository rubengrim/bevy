use super::{
    asset_binder::AssetBindings,
    blas_manager::BlasManager,
    extract_assets::ExtractedAssetEvents,
    fallback_blas_builder::{FallbackBlasNode, GpuSolariMeshPrimitive},
    fallback_tlas_builder::{
        build_fallback_tlas, FallbackTlas, FallbackTlasInstance, FallbackTlasNode,
        GpuFallbackTlasInstance,
    },
    gpu_types::{
        DirectionalLight, GpuSbvhNode, GpuSolariMaterial, LightSource, NewFallbackTlasInstance,
        SolariTriangleMeshPrimitive,
    },
    sbvh::{build_sbvh, BvhPrimitive, Sbvh},
    SolariRayAccelerationBackendType,
};
use crate::{ExtractedDirectionalLight, StandardMaterial};
use bevy_asset::{AssetId, Handle};
use bevy_ecs::{
    system::{Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_math::{bounding::Aabb3d, Mat4, Vec3, Vec4, Vec4Swizzles};
use bevy_render::{
    mesh::Mesh,
    render_resource::{binding_types::storage_buffer_read_only, encase::internal::WriteInto, *},
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
    // TODO: Needed for now because the bind group isn't properly keeping the tlas alive
    tlas: Option<TlasPackage>,
    fallback_tlas: Option<FallbackTlas>,
}

impl FromWorld for SceneBindings {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let backend_type = world.resource::<SolariRayAccelerationBackendType>();
        Self {
            bind_group_layout: match backend_type {
                SolariRayAccelerationBackendType::Hardware => render_device
                    .create_bind_group_layout(
                        "solari_scene_bind_group_layout",
                        &BindGroupLayoutEntries::sequential(
                            ShaderStages::COMPUTE,
                            (
                                storage_buffer_read_only::<u32>(false),
                                storage_buffer_read_only::<Mat4>(false),
                                storage_buffer_read_only::<GpuSolariMaterial>(false),
                                storage_buffer_read_only::<LightSource>(false),
                                storage_buffer_read_only::<DirectionalLight>(false),
                            ),
                        ),
                    ),
                SolariRayAccelerationBackendType::Software => render_device
                    .create_bind_group_layout(
                        "solari_scene_bind_group_layout",
                        &BindGroupLayoutEntries::sequential(
                            ShaderStages::COMPUTE,
                            (
                                storage_buffer_read_only::<u32>(false),
                                storage_buffer_read_only::<Mat4>(false),
                                storage_buffer_read_only::<GpuSolariMaterial>(false),
                                storage_buffer_read_only::<LightSource>(false),
                                storage_buffer_read_only::<DirectionalLight>(false),
                                storage_buffer_read_only::<FallbackTlasNode>(false),
                                storage_buffer_read_only::<GpuFallbackTlasInstance>(false),
                                storage_buffer_read_only::<u32>(false),
                                storage_buffer_read_only::<FallbackBlasNode>(false),
                                storage_buffer_read_only::<SolariTriangleMeshPrimitive>(false),
                                storage_buffer_read_only::<u32>(false),
                            ),
                        ),
                    ),
            },
            bind_group: None,
            tlas: None,
            fallback_tlas: None,
        }
    }
}

// TODO: Optimize buffer management
pub fn prepare_scene_bindings(
    mut scene_bindings: ResMut<SceneBindings>,
    directional_lights_query: Query<&ExtractedDirectionalLight>,
    asset_bindings: Res<AssetBindings>,
    scene: Res<ExtractedScene>,
    asset_events: Res<ExtractedAssetEvents>,
    blas_manager: Res<BlasManager>,
    backend_type: Res<SolariRayAccelerationBackendType>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    // Build buffer of materials
    let get_image_id = |asset_id: Option<AssetId<Image>>| match asset_id {
        Some(asset_id) => *asset_bindings
            .image_indices
            .get(&asset_id)
            .unwrap_or(&u32::MAX),
        None => u32::MAX,
    };
    let mut materials = Vec::with_capacity(asset_events.materials.len());
    let mut material_ids = HashMap::with_capacity(asset_events.materials.len());
    for (asset_id, material) in &asset_events.materials {
        material_ids.insert(*asset_id, materials.len() as u32);
        materials.push(GpuSolariMaterial {
            base_color: material.base_color.as_linear_rgba_f32(),
            emissive: material.emissive.as_linear_rgba_f32(),
            base_color_texture_id: get_image_id(material.base_color_texture),
            normal_map_texture_id: get_image_id(material.normal_map_texture),
            emissive_texture_id: get_image_id(material.emissive_texture),
            _padding: 0,
        });
    }

    let mut light_sources = Vec::new();

    // Build buffer of directional lights
    let mut directional_lights = Vec::new();
    for directional_light in &directional_lights_query {
        light_sources.push(LightSource::directional_light(
            directional_lights.len() as u32
        ));
        directional_lights.push(DirectionalLight {
            direction_to_light: directional_light.transform.back(),
            color: (directional_light.color.as_rgba_linear() * directional_light.illuminance)
                .as_linear_rgba_f32(),
        });
    }

    match *backend_type {
        SolariRayAccelerationBackendType::Hardware => {
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
            let mut object_id = 0;
            let mut transforms = Vec::with_capacity(scene.entities.len());
            let mut mesh_material_ids = Vec::with_capacity(scene.entities.len());
            for (mesh_id, material_id, transform) in &scene.entities {
                if let (Some((blas, triangle_count)), Some(mesh_id), Some(material_id)) = (
                    blas_manager.get_blas_and_triangle_count(mesh_id),
                    asset_bindings.mesh_indices.get(mesh_id),
                    material_ids.get(material_id),
                ) {
                    let transform = transform.compute_matrix();
                    transforms.push(transform);

                    // TODO: Check for ID overflow
                    mesh_material_ids.push((*mesh_id << 16) | *material_id);

                    // For emissive meshes, push each triangle to the light sources buffer
                    let material = &materials[*material_id as usize];
                    if material.emissive != [0.0; 4] || material.emissive_texture_id != u32::MAX {
                        for triangle_id in 0..*triangle_count {
                            light_sources
                                .push(LightSource::emissive_triangle(object_id, triangle_id));
                        }
                    }

                    *tlas.get_mut_single(object_id as usize).unwrap() = Some(TlasInstance::new(
                        blas,
                        tlas_transform(&transform),
                        object_id, // TODO: Max 24 bits
                        0xFF,
                    ));

                    object_id += 1;
                }
            }

            // Build TLAS
            let mut command_encoder =
                render_device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("build_tlas_command_encoder"),
                });
            command_encoder.build_acceleration_structures(&[], iter::once(&tlas));
            render_queue.submit([command_encoder.finish()]);

            // Upload GPU buffers
            let mesh_material_ids = &new_storage_buffer(
                mesh_material_ids,
                "solari_mesh_material_ids",
                &render_device,
                &render_queue,
            );
            let transforms = new_storage_buffer(
                transforms,
                "solari_transforms",
                &render_device,
                &render_queue,
            );
            let materials =
                new_storage_buffer(materials, "solari_materials", &render_device, &render_queue);
            let light_sources = new_storage_buffer(
                light_sources,
                "solari_light_sources",
                &render_device,
                &render_queue,
            );
            let directional_lights = new_storage_buffer(
                directional_lights,
                "solari_directional_lights",
                &render_device,
                &render_queue,
            );

            // Create a bind group for the created resources
            scene_bindings.bind_group = Some(render_device.create_bind_group(
                "solari_scene_bind_group",
                &scene_bindings.bind_group_layout,
                &BindGroupEntries::sequential((
                    tlas.as_binding(),
                    mesh_material_ids.binding().unwrap(),
                    transforms.binding().unwrap(),
                    materials.binding().unwrap(),
                    light_sources.binding().unwrap(),
                    directional_lights.binding().unwrap(),
                )),
            ));
            scene_bindings.tlas = Some(tlas);
        }
        SolariRayAccelerationBackendType::Software => {
            // Build each entity into the TLAS instance and push its transform/mesh_id/material_id to a GPU buffer
            let mut object_id = 0;
            let mut transforms = Vec::with_capacity(scene.entities.len());
            let mut mesh_material_ids = Vec::with_capacity(scene.entities.len());
            let mut tlas_instances = Vec::new();
            let mut tlas_bvh_primitives = Vec::new();
            let mut blas_nodes = Vec::new();
            let mut primitives = Vec::new();
            let mut primitive_indices = Vec::new();
            for (mesh_id, material_id, transform) in &scene.entities {
                if let (Some((sbvh, mesh_primitives)), Some(mesh_id), Some(material_id)) = (
                    blas_manager.get_fallback_blas_and_primitives(mesh_id),
                    asset_bindings.mesh_indices.get(mesh_id),
                    material_ids.get(material_id),
                ) {
                    let transform = transform.compute_matrix();
                    transforms.push(transform);

                    // TODO: Check for ID overflow
                    mesh_material_ids.push((*mesh_id << 16) | *material_id);

                    // For emissive meshes, push each triangle to the light sources buffer
                    let material = &materials[*material_id as usize];
                    if material.emissive != [0.0; 4] || material.emissive_texture_id != u32::MAX {
                        for triangle_id in 0..primitives.len() {
                            light_sources.push(LightSource::emissive_triangle(
                                object_id,
                                triangle_id as u32,
                            ));
                        }
                    }

                    let blas_node_offset = blas_nodes.len() as u32;
                    blas_nodes.extend(sbvh.nodes.clone());
                    let primitive_offset = primitives.len() as u32;
                    primitives.extend((*mesh_primitives).clone());
                    primitive_indices.extend(sbvh.primitive_indices.clone());

                    tlas_bvh_primitives.push(sbvh_to_bvh_primitive(sbvh, &transform, object_id));
                    tlas_instances.push(NewFallbackTlasInstance {
                        object_world: transform,
                        world_object: transform.inverse(),
                        primitive_offset,
                        primitive_count: primitives.len() as u32,
                        blas_node_offset,
                        _padding: 0,
                    });

                    object_id += 1;
                }
            }

            // let tlas = build_fallback_tlas(&tlas_instances);
            let tlas_sbvh = build_sbvh(tlas_bvh_primitives);

            // Upload GPU buffers
            let mesh_material_ids = &new_storage_buffer(
                mesh_material_ids,
                "solari_mesh_material_ids",
                &render_device,
                &render_queue,
            );
            let transforms = new_storage_buffer(
                transforms,
                "solari_transforms",
                &render_device,
                &render_queue,
            );
            let materials =
                new_storage_buffer(materials, "solari_materials", &render_device, &render_queue);
            let light_sources = new_storage_buffer(
                light_sources,
                "solari_light_sources",
                &render_device,
                &render_queue,
            );
            let directional_lights = new_storage_buffer(
                directional_lights,
                "solari_directional_lights",
                &render_device,
                &render_queue,
            );
            let tlas_nodes = new_storage_buffer(
                tlas_sbvh
                    .nodes
                    .iter()
                    .map(|n| GpuSbvhNode::from(n))
                    .collect(),
                "solari_fallback_tlas_nodes",
                &render_device,
                &render_queue,
            );
            let tlas_instances = new_storage_buffer(
                tlas_instances,
                "solari_fallback_tlas_instances",
                &render_device,
                &render_queue,
            );
            let tlas_indices = new_storage_buffer(
                tlas_sbvh.primitive_indices.clone(),
                "solari_fallback_tlas_indices",
                &render_device,
                &render_queue,
            );
            let blas_nodes = new_storage_buffer(
                blas_nodes.iter().map(|n| GpuSbvhNode::from(n)).collect(),
                "solari_blas_nodes",
                &render_device,
                &render_queue,
            );
            let primitives = new_storage_buffer(
                primitives,
                "solari_mesh_primitives",
                &render_device,
                &render_queue,
            );
            let primitive_indices = new_storage_buffer(
                primitive_indices,
                "solari_mesh_primitive_indices",
                &render_device,
                &render_queue,
            );

            // Create a bind group for the created resources
            scene_bindings.bind_group = Some(render_device.create_bind_group(
                "solari_scene_bind_group",
                &scene_bindings.bind_group_layout,
                &BindGroupEntries::sequential((
                    mesh_material_ids.binding().unwrap(),
                    transforms.binding().unwrap(),
                    materials.binding().unwrap(),
                    light_sources.binding().unwrap(),
                    directional_lights.binding().unwrap(),
                    tlas_nodes.binding().unwrap(),
                    tlas_instances.binding().unwrap(),
                    tlas_indices.binding().unwrap(),
                    blas_nodes.binding().unwrap(),
                    primitives.binding().unwrap(),
                    primitive_indices.binding().unwrap(),
                )),
            ));

            // scene_bindings.fallback_tlas = Some(tlas);
            scene_bindings.fallback_tlas = None;
        }
    }
}

fn new_storage_buffer<T: ShaderSize + WriteInto>(
    vec: Vec<T>,
    label: &'static str,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) -> StorageBuffer<Vec<T>> {
    let mut buffer = StorageBuffer::from(vec);
    buffer.set_label(Some(label));
    buffer.write_buffer(render_device, render_queue);
    buffer
}

fn tlas_transform(transform: &Mat4) -> [f32; 12] {
    transform.transpose().to_cols_array()[..12]
        .try_into()
        .unwrap()
}

// Converts an object space `Sbvh` (BLAS) instance to a world space `BvhPrimitive` used for TLAS creation
fn sbvh_to_bvh_primitive(sbvh: &Sbvh, transform: &Mat4, id: u32) -> BvhPrimitive {
    // Convert all the corners of the sbvh root node to world space.
    // It's not possible to only consider the min and max corners because the instance could be rotated.
    let b_min = sbvh.nodes[0].bounds.min;
    let b_max = sbvh.nodes[0].bounds.max;
    let corner_1 = transform
        .mul_vec4(Vec4::new(b_min.x, b_min.y, b_min.z, 1.0))
        .xyz();
    let corner_2 = transform
        .mul_vec4(Vec4::new(b_max.x, b_min.y, b_min.z, 1.0))
        .xyz();
    let corner_3 = transform
        .mul_vec4(Vec4::new(b_min.x, b_max.y, b_min.z, 1.0))
        .xyz();
    let corner_4 = transform
        .mul_vec4(Vec4::new(b_min.x, b_min.y, b_max.z, 1.0))
        .xyz();
    let corner_5 = transform
        .mul_vec4(Vec4::new(b_max.x, b_max.y, b_min.z, 1.0))
        .xyz();
    let corner_6 = transform
        .mul_vec4(Vec4::new(b_min.x, b_max.y, b_max.z, 1.0))
        .xyz();
    let corner_7 = transform
        .mul_vec4(Vec4::new(b_max.x, b_min.y, b_max.z, 1.0))
        .xyz();
    let corner_8 = transform
        .mul_vec4(Vec4::new(b_max.x, b_max.y, b_max.z, 1.0))
        .xyz();

    // Calculate bounds of world space root node
    let mut bounds_world = Aabb3d {
        min: Vec3::MAX,
        max: Vec3::MIN,
    };
    bounds_world.min = bounds_world.min.min(corner_1);
    bounds_world.min = bounds_world.min.min(corner_2);
    bounds_world.min = bounds_world.min.min(corner_3);
    bounds_world.min = bounds_world.min.min(corner_4);
    bounds_world.min = bounds_world.min.min(corner_5);
    bounds_world.min = bounds_world.min.min(corner_6);
    bounds_world.min = bounds_world.min.min(corner_7);
    bounds_world.min = bounds_world.min.min(corner_8);

    bounds_world.max = bounds_world.max.max(corner_1);
    bounds_world.max = bounds_world.max.max(corner_2);
    bounds_world.max = bounds_world.max.max(corner_3);
    bounds_world.max = bounds_world.max.max(corner_4);
    bounds_world.max = bounds_world.max.max(corner_5);
    bounds_world.max = bounds_world.max.max(corner_6);
    bounds_world.max = bounds_world.max.max(corner_7);
    bounds_world.max = bounds_world.max.max(corner_8);

    let centroid = 0.5 * (bounds_world.min + bounds_world.max);

    BvhPrimitive {
        bounds: bounds_world,
        centroid,
        primitive_id: id,
    }
}
