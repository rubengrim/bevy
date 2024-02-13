#import bevy_render::view::View

@group(2) @binding(0) var accumulation_texture: texture_storage_2d<rgba32float, read_write>;
@group(2) @binding(1) var output_texture: texture_storage_2d<rgba16float, write>;
@group(2) @binding(2) var<uniform> view: View;

@compute @workgroup_size(8, 8, 1)
fn path_trace(@builtin(global_invocation_id) global_id: vec3<u32>) {
}
