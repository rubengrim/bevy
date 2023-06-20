#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils

@compute @workgroup_size(8, 8, 1)
fn denoise_direct_diffuse_temporal(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if any(global_id.xy >= screen_size) { return; }

    let motion_vector = textureLoad(t_buffer, global_id.xy).rg;
    let uv = (vec2<f32>(global_id.xy) + 0.5) / view.viewport.zw;
    let history_uv = uv + motion_vector;
    let history_id = vec2<i32>(history_uv * view.viewport.zw);

    let history = textureLoad(direct_diffuse_denoiser_temporal_history, history_id, 0i);
    let irradiance = textureLoad(direct_diffuse, global_id.xy).rgb;

    var history_samples = history.a;

    if any(history_id < 0i) || any(history_id >= vec2<i32>(screen_size)) {
        history_samples = 0.0;
    }

    let g_buffer_previous = textureLoad(g_buffer_previous, history_id, 0i);
    let g_buffer_current = textureLoad(g_buffer, global_id.xy);
    let previous_position = depth_to_world_position(decode_g_buffer_depth(g_buffer_previous), history_uv);
    let current_position = depth_to_world_position(decode_g_buffer_depth(g_buffer_current), uv);
    let previous_normal = decode_g_buffer_world_normal(g_buffer_previous);
    let current_normal = decode_g_buffer_world_normal(g_buffer_current);

    let plane_distance = abs(dot(previous_position - current_position, current_normal));
    if plane_distance >= 0.5 {
        history_samples = 0.0;
    }

    if dot(current_normal, previous_normal) < 0.95 {
        history_samples = 0.0;
    }

    history_samples = min(history_samples + 1.0, 32.0);

    let blended_irradiance = mix(history.rgb, irradiance, 1.0 / history_samples);

    textureStore(direct_diffuse_denoised_temporal, global_id.xy, vec4(blended_irradiance, history_samples));
}

@compute @workgroup_size(8, 8, 1)
fn denoise_direct_diffuse_spatial(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if any(global_id.xy >= screen_size) { return; }

    let center_id = vec2<i32>(global_id.xy);
    var denoised_temporal = textureLoad(direct_diffuse_denoised_temporal, center_id);
    var radius = 4i - i32(round(denoised_temporal.a / 8.0));

    var irradiance = denoised_temporal.rgb;
    if radius != 0i {
        for (var xo = -radius; xo <= radius; xo++) {
            for (var yo = -radius; yo <= radius; yo++) {
                let offset = vec2<i32>(xo, yo);
                if any(offset != 0i) {
                    // TODO: Weights
                    irradiance += textureLoad(direct_diffuse_denoised_temporal, center_id + offset).rgb;
                }
            }
        }
        let size = (radius * 2i) + 1i;
        irradiance /= f32(size * size);
    }

    textureStore(direct_diffuse_denoised_spatiotemporal, center_id, vec4(irradiance, 1.0));
}
