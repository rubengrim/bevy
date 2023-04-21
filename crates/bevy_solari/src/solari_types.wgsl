#define_import_path bevy_solari::types

const PI: f32 = 3.141592653589793;

struct SolariMeshMaterial {
    mesh_index: u32,
    material_index: u32,
}

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
    vertex.local_position = packed.b0.xyz;
    vertex.local_normal = vec3(packed.b0.w, packed.b1.xy);
    vertex.uv = packed.b1.zw;
    return vertex;
}

struct SolariVertex {
    local_position: vec3<f32>,
    local_normal: vec3<f32>,
    uv: vec2<f32>,
}

const TEXTURE_MAP_NONE = 0xffffffffu;

struct SolariMaterial {
    base_color: vec4<f32>,
    base_color_map_index: u32,
    emission: vec3<f32>,
}

struct SolariSampledMaterial {
    base_color: vec3<f32>,
    emission: vec3<f32>,
}

fn sample_material(material: SolariMaterial, uv: vec2<f32>) -> SolariSampledMaterial {
    var m: SolariSampledMaterial;

    m.base_color = material.base_color.rgb;
    if material.base_color_map_index != TEXTURE_MAP_NONE {
        m.base_color *= textureSampleLevel(texture_maps[material.base_color_map_index], texture_sampler, uv, 0.0).rgb;
    }

    m.emission = material.emission;

    return m;
}

fn trace_ray(ray_origin: vec3<f32>, ray_direction: vec3<f32>) -> RayIntersection {
    let ray_flags = RAY_FLAG_NONE;
    let ray_cull_mask = 0xFFu;
    let ray_t_min = 0.001;
    let ray_t_max = 10000.0;
    let ray = RayDesc(ray_flags, ray_cull_mask, ray_t_min, ray_t_max, ray_origin, ray_direction);

    var rq: ray_query;
    rayQueryInitialize(&rq, tlas, ray);
    rayQueryProceed(&rq);
    return rayQueryGetCommittedIntersection(&rq);
}

struct SolariRayHit {
    world_position: vec3<f32>,
    world_normal: vec3<f32>,
    uv: vec2<f32>,
    material: SolariSampledMaterial,
}

fn map_ray_hit(ray_hit: RayIntersection) -> SolariRayHit {
    let mesh_material = mesh_materials[ray_hit.instance_custom_index];
    let index_buffer = &index_buffers[mesh_material.mesh_index].buffer;
    let vertex_buffer = &vertex_buffers[mesh_material.mesh_index].buffer;
    let material = materials[mesh_material.material_index];

    let indices_i = (ray_hit.primitive_index * 3u) + vec3(0u, 1u, 2u);
    let indices = vec3((*index_buffer)[indices_i.x], (*index_buffer)[indices_i.y], (*index_buffer)[indices_i.z]);
    let vertices = array<SolariVertex, 3>(unpack_vertex((*vertex_buffer)[indices.x]), unpack_vertex((*vertex_buffer)[indices.y]), unpack_vertex((*vertex_buffer)[indices.z]));
    let barycentrics = vec3(1.0 - ray_hit.barycentrics.x - ray_hit.barycentrics.y, ray_hit.barycentrics);

    let local_position = mat3x3(vertices[0].local_position, vertices[1].local_position, vertices[2].local_position) * barycentrics;
    let world_position = (ray_hit.object_to_world * vec4(local_position, 1.0)).xyz;
    let uv = mat3x2(vertices[0].uv, vertices[1].uv, vertices[2].uv) * barycentrics;
    let local_normal = mat3x3(vertices[0].local_normal, vertices[1].local_normal, vertices[2].local_normal) * barycentrics;
    let world_normal = normalize((local_normal * ray_hit.object_to_world).xyz);

    let sampled_material = sample_material(material, uv);

    return SolariRayHit(world_position, world_normal, uv, sampled_material);
}

fn rand_f(state: ptr<function, u32>) -> f32 {
    *state = *state * 747796405u + 2891336453u;
    let word = ((*state >> ((*state >> 28u) + 4u)) ^ *state) * 277803737u;
    return f32((word >> 22u) ^ word) * bitcast<f32>(0x2f800004u);
}

fn sample_cosine_hemisphere(normal: vec3<f32>, state: ptr<function, u32>) -> vec3<f32> {
    let cos_theta = 2.0 * rand_f(state) - 1.0;
    let phi = 2.0 * PI * rand_f(state);
    let sin_theta = sqrt(max(1.0 - cos_theta * cos_theta, 0.0));
    let sin_phi = sin(phi);
    let cos_phi = cos(phi);
    let unit_sphere_direction = normalize(vec3(sin_theta * cos_phi, cos_theta, sin_theta * sin_phi));
    return normal + unit_sphere_direction;
}
