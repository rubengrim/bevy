#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils
#import bevy_solari::world_cache::bindings
#import bevy_solari::world_cache::utils

@compute @workgroup_size(8, 8, 1)
fn sample_direct_diffuse(@builtin(global_invocation_id) global_id: vec3<u32>) {
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

    // TODO: ReSTIR
    let direct_light = sample_direct_lighting(pixel_world_position, pixel_world_normal, &rng);

    textureStore(direct_diffuse, global_id.xy, vec4(direct_light, 1.0));
}
