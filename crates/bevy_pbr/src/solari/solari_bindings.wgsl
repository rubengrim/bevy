#define_import_path bevy_pbr::solari::bindings

#import bevy_pbr::utils::{PI, rand_f, rand_vec2f, rand_range_u}

struct Material {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    base_color_texture_id: u32,
    normal_map_texture_id: u32,
    emissive_texture_id: u32,
    _padding: u32,
}

const TEXTURE_MAP_NONE = 0xFFFFFFFFu;

struct ResolvedMaterial {
    base_color: vec3<f32>,
    emissive: vec3<f32>,
}

struct ResolvedRayHit {
    world_position: vec3<f32>,
    world_normal: vec3<f32>,
    geometric_world_normal: vec3<f32>,
    uv: vec2<f32>,
    material: ResolvedMaterial,
    triangle_area: f32,
}

struct RayHit {
    // Is true when a hit is found within the given ray t min/max
    is_valid_hit: bool,
    object_id: u32,
    triangle_id: u32,
    // Holds u and v. w is obtained using w = 1 - u - v
    barycentrics: vec2f,
}

struct Ray {
    origin: vec3f,
    direction: vec3f,
    t: f32,
    hit_data: RayHit,
}

struct LightSource {
    kind: u32,
    id: u32,
}

const LIGHT_SOURCE_DIRECTIONAL = 0xFFFFFFFFu;

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

struct Primitive {
    p1_: vec3<f32>,
    // The BLAS builder needs to sort the primitive buffer, so we keep track of the triangle that this primitive corresponds to
    corresponding_triangle_id: u32,
    p2_: vec3<f32>,
    _padding1_: u32,
    p3_: vec3<f32>,
    _padding2_: u32,
}

struct FallbackBlasNode {
    aabb_min: vec3<f32>,
    a_or_first_primitive: u32,
    aabb_max: vec3<f32>,
    primitive_count: u32,
}

struct FallbackTlasNode {
    aabb_min: vec3<f32>,
    a_or_first_instance: u32,
    aabb_max: vec3<f32>,
    instance_count: u32,
}

struct FallbackTlasInstance {
    object_world: mat4x4f,
    world_object: mat4x4f,
    primitive_offset: u32, // Offset into `primitives`
    primitive_count: u32,
    blas_node_offset: u32, // Offset into `blas_nodes`
    _padding: u32,
}

struct VertexBuffer { vertices: array<PackedVertex> }

struct IndexBuffer { indices: array<u32> }

@group(0) @binding(0) var<storage> vertex_buffers: binding_array<VertexBuffer>;
@group(0) @binding(1) var<storage> index_buffers: binding_array<IndexBuffer>;
@group(0) @binding(2) var textures: binding_array<texture_2d<f32>>;
@group(0) @binding(3) var samplers: binding_array<sampler>;

#ifdef SOFTWARE_RAY_ACCELERATION_FALLBACK
@group(1) @binding(0) var<storage> mesh_material_ids: array<u32>;
@group(1) @binding(1) var<storage> transforms: array<mat4x4<f32>>;
@group(1) @binding(2) var<storage> materials: array<Material>;
@group(1) @binding(3) var<storage> light_sources: array<LightSource>;
@group(1) @binding(4) var<storage> directional_lights: array<DirectionalLight>;
@group(1) @binding(5) var<storage> tlas_nodes: array<FallbackTlasNode>;
@group(1) @binding(6) var<storage> tlas_instances: array<FallbackTlasInstance>;
@group(1) @binding(7) var<storage> tlas_instance_indices: array<u32>;
@group(1) @binding(8) var<storage> blas_nodes: array<FallbackBlasNode>;
@group(1) @binding(9) var<storage> primitives: array<Primitive>;
#else
@group(1) @binding(0) var tlas: acceleration_structure;
@group(1) @binding(1) var<storage> mesh_material_ids: array<u32>;
@group(1) @binding(2) var<storage> transforms: array<mat4x4<f32>>;
@group(1) @binding(3) var<storage> materials: array<Material>;
@group(1) @binding(4) var<storage> light_sources: array<LightSource>;
@group(1) @binding(5) var<storage> directional_lights: array<DirectionalLight>;
#endif

const RAY_T_MIN = 0.001;
const RAY_T_MAX = 100000.0;
const RAY_NO_CULL = 0xFFu;

fn trace_ray(ray_origin: vec3<f32>, ray_direction: vec3<f32>, ray_t_min: f32, ray_t_max: f32) -> RayHit {
    #ifdef SOFTWARE_RAY_ACCELERATION_FALLBACK
        var ray = Ray(ray_origin + 0.0001 * ray_direction, ray_direction, 1e30, RayHit());
        traverse_tlas(&ray);
        // NOTE: Checking against t_min/t_max here isn't ideal, but should work in most cases. See ray_triangle_intersect() for info.
        if ray.t > ray_t_min - 0.0001 && ray.t < ray_t_max + 0.0001 {
            ray.hit_data.is_valid_hit = true;
        } else {
            ray.hit_data.is_valid_hit = false;
        }
        return ray.hit_data;
    #else
        let ray = RayDesc(RAY_FLAG_NONE, RAY_NO_CULL, ray_t_min, ray_t_max, ray_origin, ray_direction);
        var rq: ray_query;
        rayQueryInitialize(&rq, tlas, ray);
        rayQueryProceed(&rq);
        let ray_hit_internal = rayQueryGetCommittedIntersection(&rq);
        if ray_hit_internal.kind == RAY_QUERY_INTERSECTION_NONE {
            return RayHit(false, 0u, 0u, vec2f(0.0));
        } else {
            return RayHit(true, ray_hit_internal.instance_custom_index, ray_hit_internal.primitive_index, ray_hit_internal.barycentrics);
        }
    #endif
}

// Return true if the ray didn't hit anything within ray_t_min/ray_t_max.
// If the hardware ray api is used RAY_FLAG_TERMINATE_ON_FIRST_HIT is used for performance, but in the fallback case the ray is traced exactly like in trace_ray(). 
fn trace_shadow_ray(ray_origin: vec3<f32>, ray_direction: vec3<f32>, ray_t_min: f32, ray_t_max: f32) -> bool {
    #ifdef SOFTWARE_RAY_ACCELERATION_FALLBACK
        var ray = Ray(ray_origin + 0.0001 * ray_direction, ray_direction, 1e30, RayHit());
        traverse_tlas(&ray);
        // NOTE: Checking against t_min/t_max here isn't ideal, but should work in most cases. See ray_triangle_intersect() for info.
        return ray.t > ray_t_min - 0.0001 && ray.t < ray_t_max + 0.0001;
    #else
        let ray = RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT, RAY_NO_CULL, ray_t_min, ray_t_max, ray_origin, ray_direction);
        var rq: ray_query;
        rayQueryInitialize(&rq, tlas, ray);
        rayQueryProceed(&rq);
        let ray_hit_internal = rayQueryGetCommittedIntersection(&rq);
        return ray_hit_internal.kind == RAY_QUERY_INTERSECTION_NONE;
    #endif
}

fn unpack_vertex(packed: PackedVertex) -> Vertex {
    var vertex: Vertex;
    vertex.position = packed.a.xyz;
    vertex.normal = vec3(packed.a.w, packed.b.xy);
    vertex.uv = packed.b.zw;
    vertex.tangent = packed.tangent;
    return vertex;
}

fn sample_texture(id: u32, uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(textures[id], samplers[id], uv, 0.0).rgb;
}

fn resolve_material(material: Material, uv: vec2<f32>) -> ResolvedMaterial {
    var m: ResolvedMaterial;

    m.base_color = material.base_color.rgb;
    if material.base_color_texture_id != TEXTURE_MAP_NONE {
        m.base_color *= sample_texture(material.base_color_texture_id, uv);
    }

    m.emissive = material.emissive.rgb;
    if material.emissive_texture_id != TEXTURE_MAP_NONE {
        m.emissive *= sample_texture(material.emissive_texture_id, uv);
    }

    return m;
}

fn resolve_ray_hit_inner(object_id: u32, triangle_id: u32, barycentrics_input: vec2<f32>) -> ResolvedRayHit {
    let mm_ids = mesh_material_ids[object_id];
    let mesh_id = mm_ids >> 16u;
    let material_id = mm_ids & 0xFFFFu;

    let index_buffer = &index_buffers[mesh_id].indices;
    let vertex_buffer = &vertex_buffers[mesh_id].vertices;
    let material = materials[material_id];

    let indices_i = (triangle_id * 3u) + vec3(0u, 1u, 2u);
    let indices = vec3((*index_buffer)[indices_i.x], (*index_buffer)[indices_i.y], (*index_buffer)[indices_i.z]);
    let vertices = array<Vertex, 3>(unpack_vertex((*vertex_buffer)[indices.x]), unpack_vertex((*vertex_buffer)[indices.y]), unpack_vertex((*vertex_buffer)[indices.z]));
    let barycentrics = vec3(1.0 - barycentrics_input.x - barycentrics_input.y, barycentrics_input);

    let transform = transforms[object_id];
    let local_position = mat3x3(vertices[0].position, vertices[1].position, vertices[2].position) * barycentrics;
    let world_position = (transform * vec4(local_position, 1.0)).xyz;

    let uv = mat3x2(vertices[0].uv, vertices[1].uv, vertices[2].uv) * barycentrics;

    let local_normal = mat3x3(vertices[0].normal, vertices[1].normal, vertices[2].normal) * barycentrics;
    var world_normal = normalize(mat3x3(transform[0].xyz, transform[1].xyz, transform[2].xyz) * local_normal);
    let geometric_world_normal = world_normal;
    if material.normal_map_texture_id != TEXTURE_MAP_NONE {
        let local_tangent = mat3x3(vertices[0].tangent.xyz, vertices[1].tangent.xyz, vertices[2].tangent.xyz) * barycentrics;
        let world_tangent = normalize(mat3x3(transform[0].xyz, transform[1].xyz, transform[2].xyz) * local_tangent);
        let N = world_normal;
        let T = world_tangent;
        let B = vertices[0].tangent.w * cross(N, T);
        let Nt = sample_texture(material.normal_map_texture_id, uv);
        world_normal = normalize(Nt.x * T + Nt.y * B + Nt.z * N);
    }

    let resolved_material = resolve_material(material, uv);

    let triangle_edge0 = vertices[0].position - vertices[1].position;
    let triangle_edge1 = vertices[0].position - vertices[2].position;
    let triangle_area = length(cross(triangle_edge0, triangle_edge1)) / 2.0;

    return ResolvedRayHit(world_position, world_normal, geometric_world_normal, uv, resolved_material, triangle_area);
}

fn resolve_ray_hit(ray_hit: RayHit) -> ResolvedRayHit {
    return resolve_ray_hit_inner(ray_hit.object_id, ray_hit.triangle_id, ray_hit.barycentrics);
}

// https://www.realtimerendering.com/raytracinggems/unofficial_RayTracingGems_v1.9.pdf#0004286901.INDD%3ASec28%3A303
fn sample_cosine_hemisphere(normal: vec3<f32>, state: ptr<function, u32>) -> vec3<f32> {
    let cos_theta = 1.0 - 2.0 * rand_f(state);
    let phi = 2.0 * PI * rand_f(state);
    let sin_theta = sqrt(max(1.0 - cos_theta * cos_theta, 0.0));
    let x = normal.x + sin_theta * cos(phi);
    let y = normal.y + sin_theta * sin(phi);
    let z = normal.z + cos_theta;
    return vec3(x, y, z);
}

// https://jcgt.org/published/0006/01/01/paper.pdf
fn generate_tbn(normal: vec3<f32>) -> mat3x3<f32> {
    let sign = select(-1.0, 1.0, normal.z >= 0.0);
    let a = -1.0 / (sign + normal.z);
    let b = normal.x * normal.y * a;
    let tangent = vec3(1.0 + sign * normal.x * normal.x * a, sign * b, -sign * normal.x);
    let bitangent = vec3(b, sign + normal.y * normal.y * a, -normal.y);
    return mat3x3(tangent, bitangent, normal);
}

struct LightSample {
    radiance: vec3<f32>,
    pdf: f32,
}

// https://en.wikipedia.org/wiki/Angular_diameter#Use_in_astronomy
// https://www.realtimerendering.com/raytracinggems/unofficial_RayTracingGems_v1.9.pdf#0004286901.INDD%3ASec30%3A305
fn sample_directional_light(id: u32, ray_origin: vec3<f32>, state: ptr<function, u32>) -> LightSample {
    let light = directional_lights[id];

    // Angular diameter of the sun projected onto a disk as viewed from earth = ~0.5 degrees
    // cos(0.25)
    let cos_theta_max = 0.99999048072;

    // 1 / (2 * PI * (1 - cos_theta_max))
    let pdf = 16719.2206859;

    let r = rand_vec2f(state);
    let cos_theta = (1.0 - r.x) + r.x * cos_theta_max;
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    let phi = r.y * 2.0 * PI;
    var ray_direction = vec3(vec2(cos(phi), sin(phi)) * sin_theta, cos_theta);
    ray_direction = generate_tbn(light.direction_to_light) * ray_direction;

    return LightSample(light.color.rgb, pdf);
}

fn trace_directional_light(id: u32, ray_origin: vec3<f32>, state: ptr<function, u32>) -> vec3<f32> {
    let light = directional_lights[id];

    let cos_theta_max = 0.99999048072;

    let r = rand_vec2f(state);
    let cos_theta = (1.0 - r.x) + r.x * cos_theta_max;
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    let phi = r.y * 2.0 * PI;
    var ray_direction = vec3(vec2(cos(phi), sin(phi)) * sin_theta, cos_theta);
    ray_direction = generate_tbn(light.direction_to_light) * ray_direction;

    let light_visible = f32(trace_shadow_ray(ray_origin, ray_direction, RAY_T_MIN, RAY_T_MAX));

    return light.color.rgb * light_visible;
}

fn sample_emissive_triangle(object_id: u32, triangle_id: u32, ray_origin: vec3<f32>, origin_world_normal: vec3<f32>, state: ptr<function, u32>) -> LightSample {
    // https://www.realtimerendering.com/raytracinggems/unofficial_RayTracingGems_v1.9.pdf#0004286901.INDD%3ASec22%3A297
    var barycentrics = rand_vec2f(state);
    if barycentrics.x + barycentrics.y > 1.0 { barycentrics = 1.0 - barycentrics; }
    let light_hit = resolve_ray_hit_inner(object_id, triangle_id, barycentrics);

    let pdf = 1.0 / light_hit.triangle_area;

    let light_distance = distance(ray_origin, light_hit.world_position);
    let ray_direction = (light_hit.world_position - ray_origin) / light_distance;
    let cos_theta_origin = saturate(dot(ray_direction, origin_world_normal));
    let cos_theta_light = saturate(dot(-ray_direction, light_hit.world_normal));
    let light_distance_squared = light_distance * light_distance;
    let radiance = light_hit.material.emissive.rgb * cos_theta_origin * (cos_theta_light / light_distance_squared);

    return LightSample(radiance, pdf);
}

fn trace_emissive_triangle(object_id: u32, triangle_id: u32, ray_origin: vec3<f32>, origin_world_normal: vec3<f32>, state: ptr<function, u32>) -> LightSample {
    var barycentrics = rand_vec2f(state);
    if barycentrics.x + barycentrics.y > 1.0 { barycentrics = 1.0 - barycentrics; }
    let light_hit = resolve_ray_hit_inner(object_id, triangle_id, barycentrics);

    let light_distance = distance(ray_origin, light_hit.world_position);
    let ray_direction = (light_hit.world_position - ray_origin) / light_distance;
    let cos_theta_origin = saturate(dot(ray_direction, origin_world_normal));
    let cos_theta_light = saturate(dot(-ray_direction, light_hit.world_normal));
    let light_distance_squared = light_distance * light_distance;
    let radiance = light_hit.material.emissive.rgb * cos_theta_origin * (cos_theta_light / light_distance_squared);

    let ray_t_max = light_distance - RAY_T_MIN;
    let light_visible = f32(trace_shadow_ray(ray_origin, ray_direction, RAY_T_MIN, ray_t_max));

    return radiance * light_visible;
}

fn sample_light_sources(light_id: u32, light_count: u32, ray_origin: vec3<f32>, origin_world_normal: vec3<f32>, state: ptr<function, u32>) -> LightSample {
    let light = light_sources[light_id];

    var sample: LightSample;
    if light.kind == LIGHT_SOURCE_DIRECTIONAL {
        sample = sample_directional_light(light.id, ray_origin, state);
    } else {
        sample = sample_emissive_triangle(light.id, light.kind, ray_origin, origin_world_normal, state);
    }

    sample.pdf /= f32(light_count);

    return sample;
}

fn trace_light_source(light_id: u32, ray_origin: vec3<f32>, origin_world_normal: vec3<f32>, state: ptr<function, u32>) -> vec3<f32> {
    let light = light_sources[light_id];
    if light.kind == LIGHT_SOURCE_DIRECTIONAL {
        return trace_directional_light(light.id, ray_origin, state);
    } else {
        return trace_emissive_triangle(light.id, light.kind, ray_origin, origin_world_normal, state);
    }
}

fn get_blas_node(blas_id: u32, tlas_instance_id: u32) -> FallbackBlasNode {
    let tlas_instance = tlas_instances[tlas_instance_id];
    return blas_nodes[tlas_instance.blas_node_offset + blas_id];
}

fn get_primitive(primitive_id: u32, tlas_instance_id: u32) -> Primitive {
    let tlas_instance = tlas_instances[tlas_instance_id];
    return primitives[tlas_instance.primitive_offset + primitive_id];
}

fn traverse_tlas(ray: ptr<function, Ray>) {
    // Abort on empty/invalid root node.
    if tlas_nodes[0].a_or_first_instance == 0u && tlas_nodes[0].instance_count == 0u {
        return;
    }

    var node_index = 0u;
    var stack: array<u32, 32>;
    var stack_ptr = 0;
    loop {
        let node = tlas_nodes[node_index];
        if node.instance_count > 0u { // Is leaf node
            for (var i: u32 = 0u; i < node.instance_count; i += 1u) {
                let tlas_instance_index = tlas_instance_indices[node.a_or_first_instance + i];
                traverse_blas(ray, tlas_instance_index);
            }
            if stack_ptr == 0 {
                break;
            } else {
                stack_ptr -= 1;
                node_index = stack[stack_ptr];
            }
            continue;
        }

        // Current node is an interior node, so visit child nodes in order.
        var child_a_index = node.a_or_first_instance;
        var child_b_index = child_a_index + 1u;
        let child_a = tlas_nodes[child_a_index];
        let child_b = tlas_nodes[child_b_index];
        var dist_a = ray_aabb_intersect(ray, child_a.aabb_min, child_a.aabb_max);
        var dist_b = ray_aabb_intersect(ray, child_b.aabb_min, child_b.aabb_max);
        if dist_a > dist_b {
            let d = dist_a;
            dist_a = dist_b;
            dist_b = d;
            let c = child_a_index;
            child_a_index = child_b_index;
            child_b_index = c;
        }
        if dist_a == 1e30f {
            // Missed both child nodes.
            if stack_ptr == 0 {
                break;
            } else  {
                stack_ptr -= 1;
                node_index = stack[stack_ptr];
            }
        } else {
            // Use near node next and push the far node if it's intersected by the ray.
            node_index = child_a_index;
            if dist_b != 1e30f {
                stack[stack_ptr] = child_b_index;
                stack_ptr += 1;
            }
        }
    }
}

fn traverse_blas(ray: ptr<function, Ray>, tlas_instance_id: u32) {
    // Transform ray to object/blas space.
    let tlas_instance = tlas_instances[tlas_instance_id];
    var ray_object = Ray();
    ray_object.origin = transform_position(tlas_instance.world_object, (*ray).origin);
    ray_object.direction = normalize(transform_direction(tlas_instance.world_object, (*ray).direction));
    ray_object.t = 1e30;
    ray_object.hit_data = RayHit();

    var node_index = 0u;
    var stack: array<u32, 32>;
    var stack_ptr = 0;
    loop {
        let node = get_blas_node(node_index, tlas_instance_id);
        if node.primitive_count > 0u { // Is leaf node
            for (var i: u32 = 0u; i < node.primitive_count; i += 1u) {
                ray_triangle_intersect(&ray_object, node.a_or_first_primitive + i, tlas_instance_id);
            }
            if stack_ptr == 0 {
                break;
            } else {
                stack_ptr -= 1;
                node_index = stack[stack_ptr];
            }
            continue;
        }

        var child_a_index = node.a_or_first_primitive;
        var child_b_index = child_a_index + 1u;
        let child_a = get_blas_node(child_a_index, tlas_instance_id);
        let child_b = get_blas_node(child_b_index, tlas_instance_id);
        var dist_a = ray_aabb_intersect(&ray_object, child_a.aabb_min, child_a.aabb_max);
        var dist_b = ray_aabb_intersect(&ray_object, child_b.aabb_min, child_b.aabb_max);
        if dist_a > dist_b {
            let d = dist_a;
            dist_a = dist_b;
            dist_b = d;
            let c = child_a_index;
            child_a_index = child_b_index;
            child_b_index = c;
        }
        if dist_a == 1e30f {
            // Missed both child nodes.
            if stack_ptr == 0 {
                break;
            } else  {
                stack_ptr -= 1;
                node_index = stack[stack_ptr];
            }
        } else {
            // Use near node next and push the far node if it's intersected by the ray.
            node_index = child_a_index;
            if dist_b != 1e30f {
                stack[stack_ptr] = child_b_index;
                stack_ptr += 1;
            }
        }
    }

    let hit_position_object = ray_object.origin + ray_object.t * ray_object.direction;
    let hit_position_world = transform_position(tlas_instance.object_world, hit_position_object);
    let new_t_world = length(hit_position_world - (*ray).origin);

    if new_t_world < (*ray).t {
        (*ray).t = new_t_world;
        (*ray).hit_data = ray_object.hit_data;
    }
}

fn ray_aabb_intersect(ray: ptr<function, Ray>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 {
    let t_x_1 = (aabb_min.x - (*ray).origin.x) / (*ray).direction.x;
    let t_x_2 = (aabb_max.x - (*ray).origin.x) / (*ray).direction.x;
    var t_min = min(t_x_1, t_x_2); 
    var t_max = max(t_x_1, t_x_2); 

    let t_y_1 = (aabb_min.y - (*ray).origin.y) / (*ray).direction.y;
    let t_y_2 = (aabb_max.y - (*ray).origin.y) / (*ray).direction.y;
    t_min = max(t_min, min(t_y_1, t_y_2)); 
    t_max = min(t_max, max(t_y_1, t_y_2)); 

    let t_z_1 = (aabb_min.z - (*ray).origin.z) / (*ray).direction.z;
    let t_z_2 = (aabb_max.z - (*ray).origin.z) / (*ray).direction.z;
    t_min = max(t_min, min(t_z_1, t_z_2)); 
    t_max = min(t_max, max(t_z_1, t_z_2)); 

    if (t_max >= t_min && t_min < (*ray).t && t_max > 0.0) {
        return t_min;
    } else {
        return 1e30f;
    }
}

// Moeller-Trumbore ray/triangle intersection algorithm
// Updates ray hit record if new t is smaller
fn ray_triangle_intersect(ray: ptr<function, Ray>, primitive_id: u32, tlas_instance_id: u32) {
    let primitive = get_primitive(primitive_id, tlas_instance_id);
    let edge_1 = primitive.p2_ - primitive.p1_;
    let edge_2 = primitive.p3_- primitive.p1_;
    let h = cross((*ray).direction, edge_2);
    let a = dot(edge_1, h);
    if abs(a) < 0.0001 { // Ray parallel to triangle
        return;
    }
    let f = 1.0 / a;
    let s = (*ray).origin - primitive.p1_;
    let u = f * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return;
    }
    let q = cross(s, edge_1);
    let v = f * dot((*ray).direction, q);
    if v < 0.0 || u + v > 1.0 {
        return;
    }
    let t = f * dot(edge_2, q);
    // NOTE: No check against the ray_t_min/ray_t_max supplied to trace_ray() is done here since that would require transforming the min/max values to object space.
    // That check SHOULD technically be done since now we might override an allowed hit with an unallowed but closer hit, but it's not straight forward to transform 
    // t values like that (with performance and non-uniform object scaling in mind). We'll never override an allowed hit with an unallowed but further away hit though,
    // so shadow rays aren't affected. Basically this shouldn't cause any problems
    if t < (*ray).t && t >= 0.0001 {
        (*ray).t = t;
        (*ray).hit_data.object_id = tlas_instance_id;
        (*ray).hit_data.triangle_id = primitive.corresponding_triangle_id;
        (*ray).hit_data.barycentrics = vec2f(u, v);
    }
}

fn transform_position(m: mat4x4f, p: vec3f) -> vec3f {
    let h = m * vec4f(p, 1.0);
    return h.xyz / h.w;
}

fn transform_direction(m: mat4x4f, p: vec3f) -> vec3f {
    let h = m * vec4f(p, 0.0);
    return h.xyz;
}

fn transform_normal(m: mat4x4f, p: vec3f) -> vec3f {
    let h = transpose(m) * vec4f(p, 0.0);
    return h.xyz;
}
