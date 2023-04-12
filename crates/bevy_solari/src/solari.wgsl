#import bevy_render::view

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var tlas: acceleration_structure;
@group(0) @binding(2)
var output_texture: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn solari_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    textureStore(output_texture, vec2<i32>(global_id.xy), vec4(1.0));
}
