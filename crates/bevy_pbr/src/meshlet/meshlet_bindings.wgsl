#define_import_path bevy_pbr::meshlet_bindings

#import bevy_pbr::mesh_types::Mesh
#import bevy_render::view::View

struct PackedMeshletVertex {
    a: vec4<f32>,
    b: vec4<f32>,
    tangent: vec4<f32>,
}

// TODO: Octahedral encode normal, remove tangent and derive from UV derivatives
struct MeshletVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    tangent: vec4<f32>,
}

fn unpack_meshlet_vertex(packed: PackedMeshletVertex) -> MeshletVertex {
    var vertex: MeshletVertex;
    vertex.position = packed.a.xyz;
    vertex.normal = vec3(packed.a.w, packed.b.xy);
    vertex.uv = packed.b.zw;
    vertex.tangent = packed.tangent;
    return vertex;
}

struct Meshlet {
    start_vertex_id: u32,
    start_index_id: u32,
    triangle_count: u32,
}

struct MeshletBoundingSphere {
    center: vec3<f32>,
    radius: f32,
}

struct DrawIndexedIndirect {
    vertex_count: atomic<u32>,
    instance_count: u32,
    base_index: u32,
    vertex_offset: u32,
    base_instance: u32,
}

#ifdef MESHLET_CULLING_PASS
@group(0) @binding(0) var<storage, read> meshlet_thread_meshlet_ids: array<u32>;
@group(0) @binding(1) var<storage, read> meshlet_bounding_spheres: array<MeshletBoundingSphere>;
@group(0) @binding(2) var<storage, read> meshlet_thread_instance_ids: array<u32>;
@group(0) @binding(3) var<storage, read> meshlet_instance_uniforms: array<Mesh>;
@group(0) @binding(4) var<storage, read> meshlet_view_instance_visibility: array<u32>;
@group(0) @binding(5) var<storage, read_write> meshlet_occlusion: array<atomic<u32>>; // packed bool's
@group(0) @binding(6) var<storage, read> meshlet_previous_thread_ids: array<u32>;
@group(0) @binding(7) var<storage, read> meshlet_previous_occlusion: array<u32>; // packed bool's
@group(0) @binding(8) var<uniform> view: View;
@group(0) @binding(9) var depth_pyramid: texture_2d<f32>;

fn should_cull_instance(instance_id: u32) -> bool {
    let bit_offset = instance_id % 32u;
    let packed_visibility = meshlet_view_instance_visibility[instance_id / 32u];
    return bool(extractBits(packed_visibility, bit_offset, 1u));
}

fn get_meshlet_previous_occlusion(thread_id: u32) -> bool {
    let previous_thread_id = meshlet_previous_thread_ids[thread_id];
    let packed_occlusion = meshlet_previous_occlusion[previous_thread_id / 32u];
    let bit_offset = previous_thread_id % 32u;
    return bool(extractBits(packed_occlusion, bit_offset, 1u));
}
#endif

#ifdef MESHLET_WRITE_INDEX_BUFFER_PASS
var<push_constant> second_pass: u32;
@group(0) @binding(0) var<storage, read> meshlet_occlusion: array<u32>; // packed bool's
@group(0) @binding(1) var<storage, read> meshlet_thread_meshlet_ids: array<u32>;
@group(0) @binding(2) var<storage, read> meshlet_previous_thread_ids: array<u32>;
@group(0) @binding(3) var<storage, read> meshlet_previous_occlusion: array<u32>; // packed bool's
@group(0) @binding(4) var<storage, read> meshlets: array<Meshlet>;
@group(0) @binding(5) var<storage, read_write> draw_command_buffer: DrawIndexedIndirect;
@group(0) @binding(6) var<storage, read_write> draw_index_buffer: array<u32>;

fn get_meshlet_occlusion(thread_id: u32) -> bool {
    let packed_occlusion = meshlet_occlusion[thread_id / 32u];
    let bit_offset = thread_id % 32u;
    return bool(extractBits(packed_occlusion, bit_offset, 1u));
}

fn get_meshlet_previous_occlusion(thread_id: u32) -> bool {
    let previous_thread_id = meshlet_previous_thread_ids[thread_id];
    let packed_occlusion = meshlet_previous_occlusion[previous_thread_id / 32u];
    let bit_offset = previous_thread_id % 32u;
    return bool(extractBits(packed_occlusion, bit_offset, 1u));
}
#endif

#ifdef MESHLET_VISIBILITY_BUFFER_RASTER_PASS
@group(0) @binding(0) var<storage, read> meshlet_thread_meshlet_ids: array<u32>;
@group(0) @binding(1) var<storage, read> meshlets: array<Meshlet>;
@group(0) @binding(2) var<storage, read> meshlet_indices: array<u32>; // packed u8's
@group(0) @binding(3) var<storage, read> meshlet_vertex_ids: array<u32>;
@group(0) @binding(4) var<storage, read> meshlet_vertex_data: array<PackedMeshletVertex>;
@group(0) @binding(5) var<storage, read> meshlet_thread_instance_ids: array<u32>;
@group(0) @binding(6) var<storage, read> meshlet_instance_uniforms: array<Mesh>;
@group(0) @binding(7) var<storage, read> meshlet_instance_material_ids: array<u32>;
@group(0) @binding(8) var<uniform> view: View;

fn get_meshlet_index(index_id: u32) -> u32 {
    let packed_index = meshlet_indices[index_id / 4u];
    let bit_offset = (index_id % 4u) * 8u;
    return extractBits(packed_index, bit_offset, 8u);
}
#endif

#ifdef MESHLET_MESH_MATERIAL_PASS
@group(1) @binding(0) var meshlet_visibility_buffer: texture_2d<u32>;
@group(1) @binding(1) var<storage, read> meshlet_thread_meshlet_ids: array<u32>;
@group(1) @binding(2) var<storage, read> meshlets: array<Meshlet>;
@group(1) @binding(3) var<storage, read> meshlet_indices: array<u32>; // packed u8's
@group(1) @binding(4) var<storage, read> meshlet_vertex_ids: array<u32>;
@group(1) @binding(5) var<storage, read> meshlet_vertex_data: array<PackedMeshletVertex>;
@group(1) @binding(6) var<storage, read> meshlet_thread_instance_ids: array<u32>;
@group(1) @binding(7) var<storage, read> meshlet_instance_uniforms: array<Mesh>;

fn get_meshlet_index(index_id: u32) -> u32 {
    let packed_index = meshlet_indices[index_id / 4u];
    let bit_offset = (index_id % 4u) * 8u;
    return extractBits(packed_index, bit_offset, 8u);
}
#endif
