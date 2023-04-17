#define_import_path bevy_solari::types

struct SolariMeshMaterial {
    mesh_index: u32,
    material_index: u32,
}

// TODO: These two types are temporary to work around a naga bug
struct SolariIndexBuffer {
    buffer: array<u32>,
}
struct SolariVertexBuffer {
    buffer: array<SolariPackedVertex>,
}

// The size of a vertex is 32 bytes of data
//
// The size of the SolariVertex struct when used in an
// array is padded to 64 bytes due to WGSL alignment rules
//
// This struct is properly 32 bytes
struct SolariPackedVertex {
    b0: vec4<f32>,
    b1: vec4<f32>,
}

fn unpack_vertex(packed: SolariPackedVertex) -> SolariVertex {
    var vertex: SolariVertex;
    vertex.position = packed.b0.xyz;
    vertex.normal = vec3(packed.b0.w, packed.b1.xy);
    vertex.uv = packed.b1.zw;
    return vertex;
}

struct SolariVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
}

const TEXTURE_MAP_NONE = 0xffffffffu;

struct SolariMaterial {
    base_color: vec4<f32>,
    base_color_map_index: u32,
}
