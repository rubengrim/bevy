#define_import_path bevy_pbr::solari::bindings

struct Material {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    base_color_texture_id: u32,
    normal_map_texture_id: u32,
    emissive_texture_id: u32,
    _padding: u32,
}

struct LightSource {
    kind: u32,
    id: u32,
}

struct DirectionalLight {
    direction_to_light: vec3<f32>,
    color: vec4<f32>,
}

struct PackedVertex {
    a: vec4<f32>,
    b: vec4<f32>,
    tangent: vec4<f32>,
}

struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    tangent: vec4<f32>,
}

fn unpack_vertex(packed: PackedVertex) -> Vertex {
    var vertex: Vertex;
    vertex.position = packed.a.xyz;
    vertex.normal = vec3(packed.a.w, packed.b.xy);
    vertex.uv = packed.b.zw;
    vertex.tangent = packed.tangent;
    return vertex;
}

struct VertexBuffer { vertices: array<PackedVertex> }

struct IndexBuffer { indices: array<u32> }

@group(0) @binding(0) var<storage> vertex_buffers: binding_array<VertexBuffer>;
@group(0) @binding(1) var<storage> index_buffers: binding_array<IndexBuffer>;
@group(0) @binding(2) var textures: binding_array<texture_2d<f32>>;
@group(0) @binding(3) var samplers: binding_array<sampler>;

@group(1) @binding(0) var tlas: acceleration_structure;
@group(1) @binding(1) var<storage> mesh_material_ids: array<u32>;
@group(1) @binding(2) var<storage> transforms: array<mat4x4<f32>>;
@group(1) @binding(3) var<storage> materials: array<Material>;
@group(1) @binding(4) var<storage> light_sources: array<LightSource>;
@group(1) @binding(5) var<storage> directional_lights: array<DirectionalLight>;
