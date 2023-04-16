#define_import_path bevy_solari::types

struct SolariMeshMaterial {
    mesh_index: u32,
    material_index: u32,
};

struct SolariVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
};

const TEXTURE_MAP_NONE = 0xffffffffu;

struct SolariMaterial {
    base_color: vec4<f32>,
    base_color_map_index: u32,
};
