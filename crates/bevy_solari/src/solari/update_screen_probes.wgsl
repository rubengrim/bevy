#import bevy_solari::scene_types
#import bevy_solari::scene_bindings
#import bevy_solari::utils
#import bevy_render::view
#import bevy_render::globals

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var<uniform> globals: Globals;
@group(1) @binding(2)
var g_buffer: texture_storage_2d<rgba16uint, read>;
@group(1) @binding(3)
var screen_probes: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(4)
var<storage, read_write> screen_probe_spherical_harmonics: array<SphericalHarmonicsPacked>;

var<workgroup> probe_g_pixel: vec4<u32>;
var<workgroup> probe_pixel_uv: vec2<f32>;
var<workgroup> spherical_harmonics_coefficents: array<array<vec3<f32>, 9>, 64>;

@compute @workgroup_size(8, 8, 1)
fn update_screen_probes(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
) {
    let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);
    let frame_index = globals.frame_count * 5782582u;
    var rng = pixel_index + frame_index;
    var rng2 = frame_index;

    let probe_thread_index = u32(floor(rand_f(&rng2) * 63.0));
    if local_index == probe_thread_index {
        probe_g_pixel = textureLoad(g_buffer, global_id.xy);
        probe_pixel_uv = (vec2<f32>(global_id.xy) + rand_vec2(&rng)) / view.viewport.zw;
    }
    workgroupBarrier();
    let probe_g = decode_g_buffer(probe_g_pixel, probe_pixel_uv);
    if !probe_g.ray_hit {
        return;
    }

    let octahedral_pixel_center = vec2<f32>(local_id.xy) + rand_vec2(&rng);
    let octahedral_pixel_uv = octahedral_pixel_center / 8.0;
    let octahedral_normal = octahedral_decode(octahedral_pixel_uv);

    var color = vec3(0.0);
    var throughput = vec3(1.0);
    var ray_origin = probe_g.world_position;
    var ray_direction = octahedral_normal;
    for (var i = 0u; i < 2u; i++) {
        let ray_hit = trace_ray(ray_origin, ray_direction, 0.001);
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

    // var blended_radiance = color;
    let previous_pixel = textureLoad(screen_probes, global_id.xy);
    // if previous_pixel.a == 1.0 {
    //     let current_radiance = color;
    //     let previous_radiance = previous_pixel.rgb;
    //     let l1 = dot(current_radiance, vec3(1.0 / 3.0));
    //     let l2 = dot(previous_radiance, vec3(1.0 / 3.0));
    //     var a = max(l1 - l2 - min(l1, l2), 0.0) / max(max(l1, l2), 1e-4);
    //     a = clamp(a, 0.0, 0.95);
    //     a *= a;
    //     blended_radiance = mix(current_radiance, previous_radiance, a);
    // }
    let new_color = (color + previous_pixel.a * previous_pixel.rgb) / (previous_pixel.a + 1.0);
    textureStore(screen_probes, global_id.xy, vec4(new_color, previous_pixel.a + 1.0));

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
    if local_index == 0u {
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
        screen_probe_spherical_harmonics[pixel_index / 64u] = SphericalHarmonicsPacked(
            vec4(L00, L11.x),
            vec4(L11.yz, L10.xy),
            vec4(L10.z, L1_1),
            vec4(L21, L2_1.x),
            vec4(L2_1.yz, L2_2.xy),
            vec4(L2_2.z, L20),
            L22,
        );
    }
}
