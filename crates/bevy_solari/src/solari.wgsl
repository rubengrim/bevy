#import bevy_render::view

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var tlas: acceleration_structure;

@compute @workgroup_size(8, 8, 1)
fn solari_main() {}
