#define_import_path bevy_solari::material

const TEXTURE_MAP_NONE = 0xffffffffu;

struct SolariMaterial {
    base_color: vec4<f32>,
    base_color_map_index: u32,
};
