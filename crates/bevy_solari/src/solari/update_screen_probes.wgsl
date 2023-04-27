#import bevy_solari::scene_types
#import bevy_solari::scene_bindings
#import bevy_solari::utils
#import bevy_render::view

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var accumulation_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(2)
var output_texture: texture_storage_2d<rgba16float, read_write>;
var<push_constant> previous_sample_count: f32;

fn sign_not_zero(v: vec2<f32>) -> vec2<f32> {
    return select(vec2(-1.0), vec2(1.0), v >= vec2(0.0));
}

fn octahedral_decode(e: vec2<f32>) -> vec3<f32> {
    var v = vec3(e.xy, 1.0 - abs(e.x) - abs(e.y));
    if v.z < 0.0 {
        v = vec3((1.0 - abs(v.yx)) * sign_not_zero(v.xy), v.z);
    }
    return normalize(v);
}

var<workgroup> probe_location: vec3<f32>;
var<workgroup> spherical_harmonics_coefficents: array<array<vec3<f32>, 9>, 64>;

@compute @workgroup_size(8, 8, 1)
fn update_screen_probes(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
) {
    // RNG setup
    let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);
    let frame_index = u32(previous_sample_count) * 5782582u;
    var rng = pixel_index + frame_index;

    var rng2 = frame_index;
    let probe_pixel_index = u32(floor(rand_f(&rng2) * 63.0));

    // Cast primary rays, and have one thread write the probe location for the whole 8x8 tile
    let pixel_center = vec2<f32>(global_id.xy) + 0.5;
    let primary_ray_origin = view.world_position;
    let primary_ray_direction = pixel_to_ray_direction((pixel_center + rand_vec2(&rng) - 0.5) / view.viewport.zw);
    let p_hit = trace_ray(primary_ray_origin, primary_ray_direction);
    if p_hit.kind == RAY_QUERY_INTERSECTION_NONE {
        if local_index == probe_pixel_index {
            probe_location = vec3(0.0);
        }
        textureStore(output_texture, global_id.xy, vec4(1.0));
        return;
    }
    let primary_hit = map_ray_hit(p_hit);
    if local_index == probe_pixel_index {
        probe_location = primary_hit.world_position + (primary_hit.world_normal * 0.001);
    }
    workgroupBarrier();
    if probe_location.x == 0.0 && probe_location.y == 0.0 && probe_location.z == 0.0 {
        textureStore(output_texture, global_id.xy, vec4(1.0));
        return;
    }

    // Each thread then shoots a ray through 1 pixel of the probe
    let octahedral_pixel_center = vec2<f32>(local_id.xy) + 0.5;
    let jitter = rand_vec2(&rng) - 0.5; // TODO: R2 sequence
    let octahedral_pixel_uv = (octahedral_pixel_center + jitter) / 8.0;
    let octahedral_pixel_ndc = (octahedral_pixel_uv * 2.0) - 1.0;
    let octahedral_normal = octahedral_decode(octahedral_pixel_ndc);

    // Calculate incoming radiance in that direction for the probe via path-tracing
    var color = vec3(0.0);
    var throughput = vec3(1.0);
    var ray_origin = probe_location;
    var ray_direction = octahedral_normal;
    loop {
        let ray_hit = trace_ray(ray_origin, ray_direction);
        if ray_hit.kind != RAY_QUERY_INTERSECTION_NONE {
            let ray_hit = map_ray_hit(ray_hit);

            color += ray_hit.material.emission * throughput;
            throughput *= ray_hit.material.base_color;

            let p = max(max(throughput.r, throughput.g), throughput.b);
            if rand_f(&rng) > p { break; }
            throughput *= 1.0 / p;

            ray_origin = ray_hit.world_position;
            ray_direction = sample_cosine_hemisphere(ray_hit.world_normal, &rng);
        } else { break; }
    }

    // Accumulate probe values
    let old_color = textureLoad(accumulation_texture, global_id.xy).rgb;
    let new_color = (color + previous_sample_count * old_color) / (previous_sample_count + 1.0);
    textureStore(accumulation_texture, global_id.xy, vec4(new_color, 1.0));

    // Project to spherical harmonics and then back to diffuse lighting
    // https://cseweb.ucsd.edu/~ravir/papers/envmap/envmap.pdf (eq 3, 13)
    // TODO: Use subgroup operations when wgpu/naga have support
    var x = octahedral_normal.x;
    var y = octahedral_normal.y;
    var z = octahedral_normal.z;
    var xz = x * z;
    var yz = y * z;
    var xy = x * y;
    var zz = z * z;
    var xx_yy = x * x - y * y;
    let Y00 = 0.282095;
    let Y11 = 0.488603 * x;
    let Y10 = 0.488603 * z;
    let Y1_1 = 0.488603 * y;
    let Y21 = 1.092548 * xz;
    let Y2_1 = 1.092548 * yz;
    let Y2_2 = 1.092548 * xy;
    let Y20 = 0.946176 * zz - 0.315392;
    let Y22 = 0.546274 * xx_yy;
    spherical_harmonics_coefficents[local_index][0] = new_color * Y00;
    spherical_harmonics_coefficents[local_index][1] = new_color * Y11;
    spherical_harmonics_coefficents[local_index][2] = new_color * Y10;
    spherical_harmonics_coefficents[local_index][3] = new_color * Y1_1;
    spherical_harmonics_coefficents[local_index][4] = new_color * Y21;
    spherical_harmonics_coefficents[local_index][5] = new_color * Y2_1;
    spherical_harmonics_coefficents[local_index][6] = new_color * Y2_2;
    spherical_harmonics_coefficents[local_index][7] = new_color * Y20;
    spherical_harmonics_coefficents[local_index][8] = new_color * Y22;
    workgroupBarrier();
    var L00 = vec3(0.0);
    var L11 = vec3(0.0);
    var L10 = vec3(0.0);
    var L1_1 = vec3(0.0);
    var L21 = vec3(0.0);
    var L2_1 = vec3(0.0);
    var L2_2 = vec3(0.0);
    var L20 = vec3(0.0);
    var L22 = vec3(0.0);
    for (var t = 0u; t < 64u; t++) {
        L00 += spherical_harmonics_coefficents[t][0];
        L11 += spherical_harmonics_coefficents[t][1];
        L10 += spherical_harmonics_coefficents[t][2];
        L1_1 += spherical_harmonics_coefficents[t][3];
        L21 += spherical_harmonics_coefficents[t][4];
        L2_1 += spherical_harmonics_coefficents[t][5];
        L2_2 += spherical_harmonics_coefficents[t][6];
        L20 += spherical_harmonics_coefficents[t][7];
        L22 += spherical_harmonics_coefficents[t][8];
    }
    L00 /= 64.0;
    L11 /= 64.0;
    L10 /= 64.0;
    L1_1 /= 64.0;
    L21 /= 64.0;
    L2_1 /= 64.0;
    L2_2 /= 64.0;
    L20 /= 64.0;
    L22 /= 64.0;
    let c1 = 0.429043;
    let c2 = 0.511664;
    let c3 = 0.743125;
    let c4 = 0.886227;
    let c5 = 0.247708;
    x = primary_hit.world_normal.x;
    y = primary_hit.world_normal.y;
    z = primary_hit.world_normal.z;
    xz = x * z;
    yz = y * z;
    xy = x * y;
    zz = z * z;
    xx_yy = x * x - y * y;
    let irradiance = (c1 * L22 * xx_yy) + (c3 * L20 * zz) + (c4 * L00) - (c5 * L20) + (2.0 * c1 * ((L2_2 * xy) + (L21 * xz) + (L2_1 * yz))) + (2.0 * c2 * ((L11 * x) + (L1_1 * y) + (L10 * z)));

    // Calculate final lighting
    let final_color = (primary_hit.material.base_color * irradiance) + primary_hit.material.emission;

    textureStore(output_texture, global_id.xy, vec4(final_color, 1.0));
}
