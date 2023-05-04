#define_import_path bevy_solari::utils

const PI: f32 = 3.141592653589793;

fn pixel_to_ray_direction(pixel_uv: vec2<f32>) -> vec3<f32> {
    let pixel_ndc = (pixel_uv * 2.0) - 1.0;
    let primary_ray_target = view.inverse_view_proj * vec4(pixel_ndc.x, -pixel_ndc.y, 1.0, 1.0);
    return normalize((primary_ray_target.xyz / primary_ray_target.w) - view.world_position);
}

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

fn rand_f(state: ptr<function, u32>) -> f32 {
    *state = *state * 747796405u + 2891336453u;
    let word = ((*state >> ((*state >> 28u) + 4u)) ^ *state) * 277803737u;
    return f32((word >> 22u) ^ word) * bitcast<f32>(0x2f800004u);
}

fn rand_vec2(state: ptr<function, u32>) -> vec2<f32> {
    return vec2(rand_f(state), rand_f(state));
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
    var n = vec3(f.x, f.y, 1.0 - abs(f.x) - abs(f.y));
    let t = saturate(-n.z);
    let w = select(vec2(t), vec2(-t), n.xy >= vec2(0.0));
    n = vec3(n.xy + w, n.z);
    return normalize(n);
}

struct GBufferData {
    world_position: vec3<f32>,
    world_normal: vec3<f32>,
    ray_hit: bool,
}

fn encode_g_buffer(ray_distance: f32, world_normal: vec3<f32>) -> vec4<u32> {
    let g = bitcast<u32>(ray_distance);
    let r = g >> 16u;
    let a = pack2x16float(octahedral_encode(world_normal));
    let b = a >> 16u;
    return vec4(r, g, b, a);
}

fn encode_m_buffer(material_index: u32, texture_coordinates: vec2<f32>) -> vec4<u32> {
    let g = material_index;
    let r = g >> 16u;
    let a = pack2x16float(texture_coordinates);
    let b = a >> 16u;
    return vec4(r, g, b, a);
}

fn decode_g_buffer(g_buffer_pixel: vec4<u32>, pixel_uv: vec2<f32>) -> GBufferData {
    let ray_distance = bitcast<f32>((g_buffer_pixel.r << 16u) | g_buffer_pixel.g);
    let world_normal = octahedral_decode(unpack2x16float((g_buffer_pixel.b << 16u) | g_buffer_pixel.a));

    let primary_ray_direction = pixel_to_ray_direction(pixel_uv);
    let world_position = view.world_position + (ray_distance * primary_ray_direction);

    return GBufferData(world_position, world_normal, ray_distance >= 0.0);
}

fn decode_m_buffer(m_buffer_pixel: vec4<u32>, pixel_uv: vec2<f32>) -> SolariSampledMaterial {
    let material_index = (m_buffer_pixel.r << 16u) | m_buffer_pixel.g;
    let texture_coordinates = unpack2x16float((m_buffer_pixel.b << 16u) | m_buffer_pixel.a);
    return sample_material(materials[material_index], texture_coordinates);
}

struct SphericalHarmonicsPacked {
    b0: vec4<f32>,
    b1: vec4<f32>,
    b2: vec4<f32>,
    b3: vec4<f32>,
    b4: vec4<f32>,
    b5: vec4<f32>,
    b6: vec3<f32>,
}
