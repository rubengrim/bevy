#define_import_path bevy_solari::utils

const PI: f32 = 3.141592653589793;

#ifndef EXCLUDE_VIEW
fn pixel_to_ray_direction(pixel_uv: vec2<f32>) -> vec3<f32> {
    let pixel_ndc = (pixel_uv * 2.0) - 1.0;
    let primary_ray_target = view.inverse_view_proj * vec4(pixel_ndc.x, -pixel_ndc.y, 1.0, 1.0);
    return normalize((primary_ray_target.xyz / primary_ray_target.w) - view.world_position);
}
#endif

fn trace_ray(ray_origin: vec3<f32>, ray_direction: vec3<f32>, ray_t_min: f32) -> RayIntersection {
    let ray_flags = RAY_FLAG_NONE;
    let ray_cull_mask = 0xFFu;
    let ray_t_max = 10000.0;
    let ray = RayDesc(ray_flags, ray_cull_mask, ray_t_min, ray_t_max, ray_origin, ray_direction);

    var rq: ray_query;
    rayQueryInitialize(&rq, tlas, ray);
    rayQueryProceed(&rq);
    return rayQueryGetCommittedIntersection(&rq);
}

fn sample_direct_lighting(ray_origin: vec3<f32>, origin_world_normal: vec3<f32>, origin_base_color: vec3<f32>, state: ptr<function, u32>) -> vec3<f32> {
    let light_count = arrayLength(&emissive_object_indices);
    let light_i = rand_range_u(light_count, state);
    let light_object_i = emissive_object_indices[light_i];
    let light_mm_indices = mesh_material_indices[light_object_i];
    let light_transform = emissive_object_transforms[light_i];
    let light_triangle_count = emissive_object_triangle_counts[light_i];
    let mesh_index = light_mm_indices >> 16u;
    let material_index = light_mm_indices & 0xFFFFu;
    let index_buffer = &index_buffers[mesh_index].buffer;
    let vertex_buffer = &vertex_buffers[mesh_index].buffer;
    let material = materials[material_index];
    let triangle_i = rand_range_u(light_triangle_count, state);
    let indices_i = (triangle_i * 3u) + vec3(0u, 1u, 2u);
    let indices = vec3((*index_buffer)[indices_i.x], (*index_buffer)[indices_i.y], (*index_buffer)[indices_i.z]);
    let vertices = array<SolariVertex, 3>(unpack_vertex((*vertex_buffer)[indices.x]), unpack_vertex((*vertex_buffer)[indices.y]), unpack_vertex((*vertex_buffer)[indices.z]));
    var r = rand_vec2(state);
    if r.x + r.y > 1.0 { r = 1.0 - r; }
    let barycentrics = vec3(r, 1.0 - r.x - r.y);
    let local_position = mat3x3(vertices[0].local_position, vertices[1].local_position, vertices[2].local_position) * barycentrics;
    let world_position = (light_transform * vec4(local_position, 1.0)).xyz;
    let light_distance = distance(ray_origin, world_position);

    let ray_flags = RAY_FLAG_TERMINATE_ON_FIRST_HIT;
    let ray_cull_mask = 0xFFu;
    let ray_t_min = 0.001;
    let ray_t_max = light_distance + 0.001;
    let ray_direction = (world_position - ray_origin) / light_distance;
    let ray = RayDesc(ray_flags, ray_cull_mask, ray_t_min, ray_t_max, ray_origin, ray_direction);
    var rq: ray_query;
    rayQueryInitialize(&rq, tlas, ray);
    rayQueryProceed(&rq);
    let ray_hit = rayQueryGetCommittedIntersection(&rq);

    if ray_hit.kind != RAY_QUERY_INTERSECTION_NONE && ray_hit.instance_custom_index == light_object_i {
        let local_normal = mat3x3(vertices[0].local_normal, vertices[1].local_normal, vertices[2].local_normal) * barycentrics;
        let world_normal = normalize((local_normal * ray_hit.object_to_world).xyz);

        let brdf = origin_base_color / PI;
        let le = material.emission;
        let cos_theta_origin = dot(ray_direction, origin_world_normal);
        let cos_theta_light = saturate(dot(-ray_direction, world_normal));
        let light_distance_squared = light_distance * light_distance;
        let light = brdf * le * cos_theta_origin * (cos_theta_light / light_distance_squared);

        let triangle_edge0 = vertices[0].local_position - vertices[1].local_position;
        let triangle_edge1 = vertices[0].local_position - vertices[2].local_position;
        let triangle_area = length(cross(triangle_edge0, triangle_edge1)) / 2.0;

        let probability = f32(light_count) * f32(light_triangle_count) * triangle_area;
        return light / probability;
    } else {
        return vec3(0.0);
    }
}

fn rand_u(state: ptr<function, u32>) -> u32 {
    *state = *state * 747796405u + 2891336453u;
    let word = ((*state >> ((*state >> 28u) + 4u)) ^ *state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_f(state: ptr<function, u32>) -> f32 {
    *state = *state * 747796405u + 2891336453u;
    let word = ((*state >> ((*state >> 28u) + 4u)) ^ *state) * 277803737u;
    return f32((word >> 22u) ^ word) * bitcast<f32>(0x2f800004u);
}

fn rand_vec2(state: ptr<function, u32>) -> vec2<f32> {
    return vec2(rand_f(state), rand_f(state));
}

fn rand_range_u(n: u32, state: ptr<function, u32>) -> u32 {
    return rand_u(state) % n;
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

fn octahedral_encode(v: vec3<f32>) -> vec2<f32> {
    var n = v / (abs(v.x) + abs(v.y) + abs(v.z));
    let octahedral_wrap = (1.0 - abs(n.yx)) * select(vec2(-1.0), vec2(1.0), n.xy > 0.0);
    let n_xy = select(octahedral_wrap, n.xy, n.z >= 0.0);
    return n_xy * 0.5 + 0.5;
}

fn octahedral_decode(v: vec2<f32>) -> vec3<f32> {
    let f = v * 2.0 - 1.0;
    var n = vec3(f.xy, 1.0 - abs(f.x) - abs(f.y));
    let t = saturate(-n.z);
    let w = select(vec2(t), vec2(-t), n.xy >= vec2(0.0));
    n = vec3(n.xy + w, n.z);
    return normalize(n);
}

fn encode_g_buffer(ray_distance: f32, world_normal: vec3<f32>) -> vec4<u32> {
    let rg = bitcast<u32>(ray_distance);
    let ab = pack2x16float(octahedral_encode(world_normal));

    let r = rg >> 16u;
    let b = ab >> 16u;
    let g = rg & 0xFFFFu;
    let a = ab & 0xFFFFu;
    return vec4(r, g, b, a);
}

fn encode_m_buffer(material_index: u32, texture_coordinates: vec2<f32>) -> vec4<u32> {
    let rg = material_index;
    let ab = pack2x16float(texture_coordinates);

    let r = rg >> 16u;
    let b = ab >> 16u;
    let g = rg & 0xFFFFu;
    let a = ab & 0xFFFFu;
    return vec4(r, g, b, a);
}

fn decode_g_buffer_depth(g_buffer_pixel: vec4<u32>) -> f32 {
    return bitcast<f32>((g_buffer_pixel.r << 16u) | g_buffer_pixel.g);
}

#ifndef EXCLUDE_VIEW
fn depth_to_world_position(ray_distance: f32, pixel_uv: vec2<f32>) -> vec3<f32> {
    return view.world_position + (ray_distance * pixel_to_ray_direction(pixel_uv));
}
#endif

fn decode_g_buffer_world_normal(g_buffer_pixel: vec4<u32>) -> vec3<f32> {
    return octahedral_decode(unpack2x16float((g_buffer_pixel.b << 16u) | g_buffer_pixel.a));
}

fn decode_m_buffer(m_buffer_pixel: vec4<u32>) -> SolariSampledMaterial {
    let material_index = (m_buffer_pixel.r << 16u) | m_buffer_pixel.g;
    let texture_coordinates = unpack2x16float((m_buffer_pixel.b << 16u) | m_buffer_pixel.a);
    return sample_material(materials[material_index], texture_coordinates);
}
