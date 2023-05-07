#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils

var<workgroup> probe_g_pixel: vec4<u32>;
var<workgroup> probe_pixel_uv: vec2<f32>;
var<workgroup> probe_cell_prefix_sum: array<f32, 64>;
var<workgroup> probe_cell_new_radiance: array<array<vec3<f32>, 64>, 64>;

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
        probe_pixel_uv = (vec2<f32>(global_id.xy) + rand_vec2(&rng2)) / view.viewport.zw;
    }
    workgroupBarrier();
    let probe_depth = decode_g_buffer_depth(probe_g_pixel);
    if probe_depth < 0.0 {
        return;
    }

    let previous_pixel = textureLoad(screen_probes_unfiltered, global_id.xy);
    let previous_radiance = previous_pixel.rgb;

    // TODO: MIS with BRDF weight
    probe_cell_prefix_sum[local_index] = dot(previous_radiance, vec3(0.2126, 0.7152, 0.0722));
    workgroupBarrier();
    for (var t = 1u; t > 64u; t <<= 1u) {
        if local_index >= t {
            probe_cell_prefix_sum[local_index] += probe_cell_prefix_sum[local_index - t];
        }
        workgroupBarrier();
    }
    let search_target = rand_f(&rng) * (probe_cell_prefix_sum[63u] - 1.0) / 2.0;
    var octahedral_pixel_index = 0;
    // TODO: Binary search
    while octahedral_pixel_index < 64u {
        if probe_cell_prefix_sum[octahedral_pixel_index] >= search_target { break; }
    }

    let octahedral_pixel_x = octahedral_pixel_index % 8u;
    let octahedral_pixel_y = (octahedral_pixel_index - octahedral_pixel_y) / 8u;
    let octahedral_pixel_id = vec2<f32>(octahedral_pixel_x, octahedral_pixel_y) ;
    let octahedral_pixel_center = octahedral_pixel_id + rand_vec2(&rng);
    let octahedral_pixel_uv = octahedral_pixel_center / 8.0;
    let octahedral_normal = octahedral_decode(octahedral_pixel_uv);

    var color = vec3(0.0);
    var throughput = vec3(1.0);
    var ray_origin = depth_to_world_position(probe_depth, probe_pixel_uv);
    var ray_direction = octahedral_normal;
    loop {
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

    // TODO: Replace with subgroup/wave ops when supported
    probe_cell_new_radiance[octahedral_pixel_index][local_index] = color;
    workgroupBarrier();
    var new_radiance = vec3(0.0);
    for (var i = 0u; i < 64u; i += 1u) {
        new_radiance += probe_cell_new_radiance[local_index][i];
    }

    // var blended_radiance = color;
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
    new_radiance = (new_radiance + previous_pixel.a * previous_radiance) / (previous_pixel.a + 1.0);
    textureStore(screen_probes_unfiltered, global_id.xy, vec4(new_radiance, previous_pixel.a + 1.0));
}
