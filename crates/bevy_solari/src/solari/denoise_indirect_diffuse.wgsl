#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils

@compute @workgroup_size(8, 8, 1)
fn denoise_indirect_diffuse_temporal(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if global_id.x >= screen_size.x || global_id.y >= screen_size.y {
        return;
    }

    let motion_vector = textureLoad(t_buffer, global_id.xy).rg;
    let irradiance = textureLoad(indirect_diffuse, global_id.xy);
    // TODO

    textureStore(indirect_diffuse_denoised_temporal, global_id.xy, irradiance);
}

@compute @workgroup_size(8, 8, 1)
fn denoise_indirect_diffuse_spatial(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if global_id.x >= screen_size.x || global_id.y >= screen_size.y {
        return;
    }

    let irradiance = textureLoad(indirect_diffuse_denoised_temporal, global_id.xy);
    // TODO

    textureStore(indirect_diffuse_denoised_spatiotemporal, global_id.xy, irradiance);
}
