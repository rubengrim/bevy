use bevy_asset::AssetId;
use bevy_math::{Mat4, Vec3};
use bevy_render::{color::Color, render_resource::ShaderType, texture::Image};

use super::sbvh::SbvhNode;

pub struct SolariMaterial {
    pub base_color: Color,
    pub base_color_texture: Option<AssetId<Image>>,
    pub normal_map_texture: Option<AssetId<Image>>,
    pub emissive: Color,
    pub emissive_texture: Option<AssetId<Image>>,
}

#[derive(ShaderType)]
pub struct GpuSolariMaterial {
    pub base_color: [f32; 4],
    pub emissive: [f32; 4],
    pub base_color_texture_id: u32,
    pub normal_map_texture_id: u32,
    pub emissive_texture_id: u32,
    pub _padding: u32,
}

#[derive(ShaderType)]
pub struct DirectionalLight {
    pub direction_to_light: Vec3,
    pub color: [f32; 4],
}

#[derive(ShaderType)]
pub struct LightSource {
    kind: u32,
    id: u32,
}

impl LightSource {
    pub fn directional_light(id: u32) -> Self {
        Self { kind: u32::MAX, id }
    }

    pub fn emissive_triangle(object_id: u32, triangle_id: u32) -> Self {
        assert_ne!(triangle_id, u32::MAX);
        Self {
            kind: triangle_id,
            id: object_id,
        }
    }
}

#[derive(ShaderType, Clone)]
pub struct SolariTriangleMeshPrimitive {
    pub p1: Vec3,
    pub _padding1: u32,
    pub p2: Vec3,
    pub _padding2: u32,
    pub p3: Vec3,
    pub _padding3: u32,
}

#[derive(ShaderType)]
pub struct GpuSbvhNode {
    pub aabb_min: Vec3,
    // Index to child a or to first primitive (triangle).
    pub a_or_first_primitive: u32,
    pub aabb_max: Vec3,
    // > 0 indicates leaf and a_or_tri contains index to first tri. Otherwise a_or_tri contains index to child node a.
    pub primitive_count: u32,
}

impl From<&SbvhNode> for GpuSbvhNode {
    fn from(n: &SbvhNode) -> Self {
        Self {
            aabb_min: n.bounds.min,
            a_or_first_primitive: if n.primitive_count > 0 {
                n.first_primitive
            } else {
                n.child_a_idx
            },
            aabb_max: n.bounds.max,
            primitive_count: n.primitive_count,
        }
    }
}

#[derive(Default, ShaderType, Clone, Debug)]
pub struct NewFallbackTlasInstance {
    pub object_world: Mat4,
    pub world_object: Mat4,
    pub primitive_offset: u32,
    pub primitive_count: u32,
    pub blas_node_offset: u32,
    pub _padding: u32,
}
