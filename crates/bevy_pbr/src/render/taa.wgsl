struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

@vertex
fn fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.uv.x = select(0.0, 2.0, vertex_index == u32(2));
    out.uv.y = select(0.0, 2.0, vertex_index == u32(1));
    out.position = vec4<f32>(
        ((out.uv * vec2<f32>(2.0, -2.0)) + vec2<f32>(-1.0, 1.0)),
        1.0,
        1.0
    );
    return out;
}

// ----------------------------------------------------------------------------

#import bevy_pbr::mesh_view_types

@group(0) @binding(0) var view_target: texture_2d<f32>;
@group(0) @binding(1) var taa_accumulation: texture_2d<f32>;
@group(0) @binding(2) var velocity: texture_2d<f32>;
@group(0) @binding(3) var<uniform> view: View;
@group(0) @binding(4) var f_sampler: sampler;

@fragment
fn taa(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // Fetch the current sample
    let current_color = textureSample(view_target, f_sampler, uv).rgb;

    // Reproject to find the equivalent sample from the past
    let current_velocity = textureSample(velocity, f_sampler, uv).rg;
    let previous_uv = uv + current_velocity;
    let previous_color = textureSample(taa_accumulation, f_sampler, previous_uv).rgb;

    // Neighborhood clamping (3x3)
    // TODO: Unroll loop by hand
    var min_color = vec3(9999.0);
    var max_color = vec3(-9999.0);
    for (var x = -1; x <= 1; x += 1) {
        for (var y = -1; y <= 1; y += 1) {
            let uv = uv + (vec2(f32(x), f32(y)) / view.viewport.zw);
            let color = textureSample(view_target, f_sampler, uv).rgb;
            min_color = min(min_color, color);
            max_color = max(max_color, color);
        }
    }
    let previous_color = clamp(previous_color, min_color, max_color);

    // Blend current and previous samples
    let output = (current_color * 0.1) + (previous_color * 0.9);

    return vec4<f32>(output, 1.0);
}

// ----------------------------------------------------------------------------

@group(0) @binding(0) var taa_output: texture_2d<f32>;
@group(0) @binding(1) var c_sampler: sampler;

struct BlitOutput {
    @location(0) r0: vec4<f32>,
    @location(1) r1: vec4<f32>,
}

@fragment
fn blit(@location(0) uv: vec2<f32>) -> BlitOutput {
    var out: BlitOutput;
    out.r0 = textureSample(taa_output, c_sampler, uv);
    out.r1 = textureSample(taa_output, c_sampler, uv);
    return out;
}
