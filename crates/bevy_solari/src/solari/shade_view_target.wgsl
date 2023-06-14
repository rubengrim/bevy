#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils
#import bevy_solari::world_cache::bindings
#import bevy_solari::world_cache::utils

@compute @workgroup_size(8, 8, 1)
fn shade_view_target(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(num_workgroups) workgroup_count: vec3<u32>,
) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if any(global_id.xy >= screen_size) { return; }

    let pixel_index = global_id.x + global_id.y * screen_size.x;
    let frame_index = globals.frame_count * 5782582u;
    var rng = pixel_index + frame_index;

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

    // TODO: Use spatiotemporal blue noise for direct light rng
    let direct_light = sample_direct_lighting(pixel_world_position, pixel_world_normal, &rng);
    let indirect_light = textureLoad(indirect_diffuse_denoised_spatiotemporal, global_id.xy).rgb;

    var final_color = material.emission;
    final_color += direct_light * (material.base_color / PI);
    final_color += indirect_light * material.base_color;

#ifdef DEBUG_VIEW_DEPTH
    final_color = vec3(pixel_depth * pixel_depth / MAX_DEPTH);
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
