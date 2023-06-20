#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils

@compute @workgroup_size(8, 8, 1)
fn sample_direct_diffuse(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if any(global_id.xy >= screen_size) { return; }

    let light_count = arrayLength(&emissive_object_indices);
    if light_count == 0u {
        textureStore(direct_diffuse, global_id.xy, vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }

    let pixel_index = global_id.x + global_id.y * screen_size.x;
    let frame_index = globals.frame_count * 5782582u;
    var rng = pixel_index + frame_index;

    let g_buffer_pixel = textureLoad(g_buffer, global_id.xy);
    let pixel_depth = decode_g_buffer_depth(g_buffer_pixel);
    if pixel_depth < 0.0 {
        textureStore(direct_diffuse, global_id.xy, vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }
    let pixel_id = vec2<f32>(global_id.xy) + 0.5;
    let pixel_world_position = depth_to_world_position(pixel_depth, pixel_id / view.viewport.zw);
    let pixel_world_normal = decode_g_buffer_world_normal(g_buffer_pixel);
    let pixel_material = decode_m_buffer(textureLoad(m_buffer, global_id.xy));
    let pixel_brdf = pixel_material.base_color / PI;

    var reservoir_unshadowed_light = vec3(0.0);
    var reservoir_light_position = vec3(0.0);
    var reservoir_target_pdf = 0.0;
    var reservoir_weight = 0.0;
    for (var m = 0u; m < 32u; m++) {
        let sample = sample_unshadowed_direct_lighting(pixel_world_position, pixel_world_normal, light_count, &rng);
        let target_pdf = dot(pixel_brdf * sample.light, vec3(0.2126729, 0.7151522, 0.0721750));
        let sample_weight = target_pdf * sample.inverse_pdf;
        reservoir_weight += sample_weight;
        if rand_f(&rng) < sample_weight / reservoir_weight {
            reservoir_unshadowed_light = sample.light;
            reservoir_light_position = sample.world_position;
            reservoir_target_pdf = target_pdf;
        }
    }

    var direct_light = (reservoir_unshadowed_light * reservoir_weight) / (reservoir_target_pdf * 32.0);
    direct_light *= trace_light_visibility(pixel_world_position, reservoir_light_position, distance(pixel_world_position, reservoir_light_position));
    direct_light = max(vec3(0.0), direct_light);

    textureStore(direct_diffuse, global_id.xy, vec4(direct_light, 1.0));
}
