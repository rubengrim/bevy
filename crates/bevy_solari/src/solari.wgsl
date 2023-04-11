#import bevy_render::view

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var tlas: acceleration_structure;
@group(0) @binding(2)
var output_texture: texture_storage_2d<f32, write>;

@compute @workgroup_size(8, 8, 1)
fn solari_main() {}
