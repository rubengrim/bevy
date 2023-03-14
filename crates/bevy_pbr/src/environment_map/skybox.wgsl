#import bevy_render::view
#import bevy_core_pipeline::fullscreen_vertex_shader

@group(0) @binding(1)
var environment_map: texture_cube<f32>;
@group(0) @binding(2)
var environment_map_sampler: sampler;
@group(0) @binding(3)
var<uniform> view: View;

@fragment
fn background(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(environment_map, environment_map_sampler, in.uv);
}
