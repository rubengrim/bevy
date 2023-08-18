#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils

@compute @workgroup_size(8, 8, 1)
fn sample_direct_diffuse(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if any(global_id.xy >= screen_size) { return; }

    let pixel_index = global_id.x + global_id.y * screen_size.x;
    let frame_index = uniforms.frame_count * 5782582u;
    var rng = pixel_index + frame_index;

    var reservoir = Reservoir(vec3(0.0), vec3(0.0), 0.0, 0.0, 0.0, 0u);

    let light_count = arrayLength(&emissive_object_indices);
    if light_count == 0u {
        direct_diffuse_reservoirs[pixel_index] = reservoir;
        textureStore(direct_diffuse, global_id.xy, vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }

    let g_buffer_pixel = textureLoad(g_buffer, global_id.xy);
    let pixel_depth = decode_g_buffer_depth(g_buffer_pixel);
    if pixel_depth < 0.0 {
        direct_diffuse_reservoirs[pixel_index] = reservoir;
        textureStore(direct_diffuse, global_id.xy, vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }
    let pixel_id = vec2<f32>(global_id.xy) + 0.5;
    let pixel_world_position = depth_to_world_position(pixel_depth, pixel_id / view.viewport.zw);
    let pixel_world_normal = decode_g_buffer_world_normal(g_buffer_pixel);
    let pixel_material = decode_m_buffer(textureLoad(m_buffer, global_id.xy));
    let pixel_brdf = pixel_material.base_color / PI;

    var temporal_combine = true;
    let motion_vector = textureLoad(t_buffer, global_id.xy).rg;
    let uv = (vec2<f32>(global_id.xy) + 0.5) / view.viewport.zw;
    let history_uv = uv + motion_vector;
    let history_id = vec2<i32>(history_uv * view.viewport.zw);
    let history = textureLoad(direct_diffuse_denoiser_temporal_history, history_id, 0i);
    if any(history_id < 0i) || any(history_id >= vec2<i32>(screen_size)) {
        temporal_combine = false;
    }
    let g_buffer_previous = textureLoad(g_buffer_previous, history_id, 0i);
    let previous_position = depth_to_world_position(decode_g_buffer_depth(g_buffer_previous), history_uv);
    let previous_normal = decode_g_buffer_world_normal(g_buffer_previous);
    let plane_distance = abs(dot(previous_position - pixel_world_position, pixel_world_normal));
    if plane_distance >= 0.5 {
        temporal_combine = false;
    }
    if dot(pixel_world_normal, previous_normal) < 0.95 {
        temporal_combine = false;
    }

    reservoir.sample_count = 32u;
    for (var m = 0u; m < 32u; m++) {
        let sample = sample_unshadowed_direct_lighting(pixel_world_position, pixel_world_normal, light_count, &rng);
        let target_pdf = dot(pixel_brdf * sample.light, vec3(0.2126729, 0.7151522, 0.0721750));
        let sample_weight = target_pdf * sample.inverse_pdf;
        reservoir.weight += sample_weight;
        if rand_f(&rng) < sample_weight / reservoir.weight {
            reservoir.unshadowed_light = sample.light;
            reservoir.light_position = sample.world_position;
            reservoir.target_pdf = target_pdf;
        }
    }
    reservoir.W = reservoir.weight / (reservoir.target_pdf * f32(reservoir.sample_count));

    var direct_light = vec3(0.0);
    var direct_light_weight = 1.0;

    // TODO: This is broken
    // if temporal_combine {
    //     reservoir.W *= trace_light_visibility(pixel_world_position, reservoir.light_position, distance(pixel_world_position, reservoir.light_position));

    //     direct_light += reservoir.unshadowed_light * reservoir.W;
    //     direct_light_weight = 0.5;

    //     var history_reservoir = direct_diffuse_reservoirs_history[u32(history_id.x) + u32(history_id.y) * screen_size.x];
    //     history_reservoir.sample_count = min(history_reservoir.sample_count, 32u * 20u);
    //     let history_weight = history_reservoir.target_pdf * history_reservoir.W * f32(history_reservoir.sample_count);

    //     reservoir.weight += history_weight;
    //     reservoir.sample_count += history_reservoir.sample_count;
    //     if rand_f(&rng) < history_weight / reservoir.weight {
    //         reservoir.unshadowed_light = history_reservoir.unshadowed_light;
    //         reservoir.light_position = history_reservoir.light_position;
    //         reservoir.target_pdf = history_reservoir.target_pdf;
    //     }
    //     reservoir.W = reservoir.weight / (reservoir.target_pdf * f32(reservoir.sample_count));
    // }

    direct_light += reservoir.unshadowed_light * reservoir.W;
    direct_light *= trace_light_visibility(pixel_world_position, reservoir.light_position, distance(pixel_world_position, reservoir.light_position));
    direct_light *= direct_light_weight;
    direct_light = max(vec3(0.0), direct_light);

    direct_diffuse_reservoirs[pixel_index] = reservoir;
    textureStore(direct_diffuse, global_id.xy, vec4(direct_light, 1.0));
}
