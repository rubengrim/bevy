#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils
#import bevy_solari::world_cache::bindings
#import bevy_solari::world_cache::utils

// TODO: Validate neighbor probe exists
// TODO: Change screen space distance to depend on camera zoom
fn interpolate_probe(
    irradiance_total: ptr<function, vec3<f32>>,
    weight_total: ptr<function, f32>,
    pixel_id: vec2<f32>,
    pixel_world_position: vec3<f32>,
    pixel_world_normal: vec3<f32>,
    probe_count_x: i32,
    probe_id: vec2<i32>,
    probe_thread_id: vec2<i32>,
) {
    let probe_pixel_id = probe_thread_id + (8i * probe_id);
    let probe_pixel_id_center = vec2<f32>(probe_pixel_id) + 0.5;
    let probe_depth = decode_g_buffer_depth(textureLoad(g_buffer, probe_pixel_id));
    let probe_world_position = depth_to_world_position(probe_depth, probe_pixel_id_center / view.viewport.zw);
    let plane_dist = abs(dot(probe_world_position - pixel_world_position, pixel_world_normal));
    if plane_dist > 0.03 {
        return;
    }

    let c1 = 0.429043;
    let c2 = 0.511664;
    let c3 = 0.743125;
    let c4 = 0.886227;
    let c5 = 0.247708;
    let x = pixel_world_normal.x;
    let y = pixel_world_normal.y;
    let z = pixel_world_normal.z;
    let xz = x * z;
    let yz = y * z;
    let xy = x * y;
    let zz = z * z;
    let xx_yy = x * x - y * y;

    let sh_index = probe_id.x + probe_id.y * probe_count_x;
    let sh = screen_probe_spherical_harmonics[sh_index];
    let L00 = sh.b0.xyz;
    let L11 = vec3(sh.b0.w, sh.b1.xy);
    let L10 = vec3(sh.b1.zw, sh.b2.x);
    let L1_1 = sh.b2.yzw;
    let L21 = sh.b3.xyz;
    let L2_1 = vec3(sh.b3.w, sh.b4.xy);
    let L2_2 = vec3(sh.b4.zw, sh.b5.x);
    let L20 = sh.b5.yzw;
    let L22 = sh.b6;
    var irradiance = (c1 * L22 * xx_yy) + (c3 * L20 * zz) + (c4 * L00) - (c5 * L20) + (2.0 * c1 * ((L2_2 * xy) + (L21 * xz) + (L2_1 * yz))) + (2.0 * c2 * ((L11 * x) + (L1_1 * y) + (L10 * z)));

    let screen_distance = distance(probe_pixel_id_center, pixel_id);
    let screen_distance_weight = smoothstep(22.0, 0.0, screen_distance);
    irradiance *= screen_distance_weight;
    *weight_total += screen_distance_weight;

    *irradiance_total += irradiance;
}

@compute @workgroup_size(8, 8, 1)
fn shade_view_target(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(num_workgroups) workgroup_count: vec3<u32>,
) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if global_id.x >= screen_size.x || global_id.y >= screen_size.y {
        return;
    }

    let probe_index = workgroup_id.x + workgroup_id.y * workgroup_count.x;
    let pixel_index = global_id.x + global_id.y * screen_size.x;
    let frame_index = globals.frame_count * 5782582u;
    var rng = pixel_index + frame_index;
    var rng2 = frame_index;

    let probe_thread_index = u32(floor(rand_f(&rng2) * 63.0));
    let probe_thread_x = probe_thread_index % 8u;
    let probe_thread_y = (probe_thread_index - probe_thread_x) / 8u;
    let probe_thread_id = vec2<i32>(vec2(probe_thread_x, probe_thread_y));

    let g_buffer_pixel = textureLoad(g_buffer, global_id.xy);
    let pixel_depth = decode_g_buffer_depth(g_buffer_pixel);
    if pixel_depth < 0.0 {
        textureStore(view_target, global_id.xy, vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }
    let pixel_id = vec2<f32>(global_id.xy) + 0.5;
    let pixel_world_position = depth_to_world_position(pixel_depth, pixel_id / view.viewport.zw);
    let pixel_world_normal = decode_g_buffer_world_normal(g_buffer_pixel);
    let material = decode_m_buffer(textureLoad(m_buffer, global_id.xy));

    let direct_light = sample_direct_lighting(pixel_world_position, pixel_world_normal, &rng);

    var indirect_light = vec3(0.0);
    var weight = 0.0;
    // TODO: Cancel jitter if outside pixel plane
    // TODO: Jitter size?
    let pixel_id_jittered = pixel_id + (rand_vec2(&rng) - 0.5);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(-1i, 1i), probe_thread_id);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(0i, 1i), probe_thread_id);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(1i, 1i), probe_thread_id);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(-1i, 0i), probe_thread_id);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(0i, 0i), probe_thread_id);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(1i, 0i), probe_thread_id);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(-1i, -1i), probe_thread_id);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(0i, -1i), probe_thread_id);
    interpolate_probe(&indirect_light, &weight, pixel_id_jittered, pixel_world_position, pixel_world_normal, i32(workgroup_count.x), vec2<i32>(workgroup_id.xy) + vec2(1i, -1i), probe_thread_id);
    if weight == 0.0 {
        weight = 9.0;
    }
    indirect_light /= weight;

    var final_color = material.emission;
    final_color += direct_light * (material.base_color / PI);
    final_color += indirect_light * material.base_color;

#ifdef DEBUG_VIEW_DEPTH
    final_color = vec3(pixel_depth * pixel_depth / 1000.0);
#endif
#ifdef DEBUG_VIEW_WORLD_NORMALS
    final_color = pixel_world_normal * 0.5 + 0.5;
#endif
#ifdef DEBUG_VIEW_MOTION_VECTORS
    let t_buffer_pixel = textureLoad(t_buffer, global_id.xy);
    // TODO: Better visualization
    final_color = vec3(abs(t_buffer_pixel.rg), 0.0);
#endif
#ifdef DEBUG_VIEW_BASE_COLORS
    final_color = material.base_color;
#endif
#ifdef DEBUG_VIEW_WORLD_CACHE_IRRADIANCE
    let world_cache_key = compute_key(pixel_world_position, pixel_world_normal);
    final_color = world_cache_irradiance[world_cache_key].rgb;
#endif
#ifdef DEBUG_VIEW_SCREEN_PROBES_UNFILTERED
    final_color = textureLoad(screen_probes_unfiltered, global_id.xy).rgb;
#endif
#ifdef DEBUG_VIEW_SCREEN_PROBES_FILTERED
    final_color = textureLoad(screen_probes_filtered, global_id.xy).rgb;
#endif
#ifdef DEBUG_VIEW_DIRECT_LIGHT
    final_color = direct_light;
#endif
#ifdef DEBUG_VIEW_INDIRECT_LIGHT
    final_color = indirect_light;
#endif

    textureStore(view_target, global_id.xy, vec4(final_color, 1.0));
    // TODO: Enable TAA
    // textureStore(view_target_other, global_id.xy, vec4(final_color, 1.0));
}
