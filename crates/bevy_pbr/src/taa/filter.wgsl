#import bevy_core_pipeline::fullscreen_vertex_shader

@group(0) @binding(0) var history: texture_2d<f32>;
@group(0) @binding(1) var linear_sample: sampler;

@fragment
fn digital_filter(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(history));
    let texel_size = 1.0 / texture_size;

#ifdef HORIZONTAL
    let o1 = vec2(texel_size.x * 1.0, 0.0);
    let o2 = vec2(texel_size.x * 2.0, 0.0);
    let o3 = vec2(texel_size.x * 3.0, 0.0);
    let o4 = vec2(texel_size.x * 4.0, 0.0);
    let o5 = vec2(texel_size.x * 5.0, 0.0);
    let o6 = vec2(texel_size.x * 6.0, 0.0);
    let o7 = vec2(texel_size.x * 7.0, 0.0);
    let o8 = vec2(texel_size.x * 8.0, 0.0);
    let o9 = vec2(texel_size.x * 9.0, 0.0);
    let o10 = vec2(texel_size.x * 10.0, 0.0);
    let o11 = vec2(texel_size.x * 11.0, 0.0);
    let o12 = vec2(texel_size.x * 12.0, 0.0);
    let o13 = vec2(texel_size.x * 13.0, 0.0);
    let o14 = vec2(texel_size.x * 14.0, 0.0);
    let o15 = vec2(texel_size.x * 15.0, 0.0);
#else
    let o1 = vec2(0.0, texel_size.y * 1.0);
    let o2 = vec2(0.0, texel_size.y * 2.0);
    let o3 = vec2(0.0, texel_size.y * 3.0);
    let o4 = vec2(0.0, texel_size.y * 4.0);
    let o5 = vec2(0.0, texel_size.y * 5.0);
    let o6 = vec2(0.0, texel_size.y * 6.0);
    let o7 = vec2(0.0, texel_size.y * 7.0);
    let o8 = vec2(0.0, texel_size.y * 8.0);
    let o9 = vec2(0.0, texel_size.y * 9.0);
    let o10 = vec2(0.0, texel_size.y * 10.0);
    let o11 = vec2(0.0, texel_size.y * 11.0);
    let o12 = vec2(0.0, texel_size.y * 12.0);
    let o13 = vec2(0.0, texel_size.y * 13.0);
    let o14 = vec2(0.0, texel_size.y * 14.0);
    let o15 = vec2(0.0, texel_size.y * 15.0);
#endif

    let weights = array(
        2.706139173000214,
        -1.236071432908206,
        0.554861954173700,
        -0.248975967923027,
        0.111718758307653,
        -0.050129651424938,
        0.022493822692047,
        -0.010093269050709,
        0.004528980312706,
        -0.002032212018705,
        0.000911879806009,
        -0.000409172258088,
        0.000183600882140,
        -0.000082384089479,
        0.000036966806042,
        -0.000016587483792
    );

    let original_color = textureSample(history, linear_sample, uv);
    var new_color = vec4(original_color.rgb, 1.0) * weights[0];

    let a = textureSample(history, linear_sample, uv + o1);
    let b = textureSample(history, linear_sample, uv - o1);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[1];

    let a = textureSample(history, linear_sample, uv + o2);
    let b = textureSample(history, linear_sample, uv - o2);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[2];

    let a = textureSample(history, linear_sample, uv + o3);
    let b = textureSample(history, linear_sample, uv - o3);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[3];

    let a = textureSample(history, linear_sample, uv + o4);
    let b = textureSample(history, linear_sample, uv - o4);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[4];

    let a = textureSample(history, linear_sample, uv + o5);
    let b = textureSample(history, linear_sample, uv - o5);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[5];

    let a = textureSample(history, linear_sample, uv + o6);
    let b = textureSample(history, linear_sample, uv - o6);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[6];

    let a = textureSample(history, linear_sample, uv + o7);
    let b = textureSample(history, linear_sample, uv - o7);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[7];

    let a = textureSample(history, linear_sample, uv + o8);
    let b = textureSample(history, linear_sample, uv - o8);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[8];

    let a = textureSample(history, linear_sample, uv + o9);
    let b = textureSample(history, linear_sample, uv - o9);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[9];

    let a = textureSample(history, linear_sample, uv + o10);
    let b = textureSample(history, linear_sample, uv - o10);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[10];

    let a = textureSample(history, linear_sample, uv + o11);
    let b = textureSample(history, linear_sample, uv - o11);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[11];

    let a = textureSample(history, linear_sample, uv + o12);
    let b = textureSample(history, linear_sample, uv - o12);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[12];

    let a = textureSample(history, linear_sample, uv + o13);
    let b = textureSample(history, linear_sample, uv - o13);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[13];

    let a = textureSample(history, linear_sample, uv + o14);
    let b = textureSample(history, linear_sample, uv - o14);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[14];

    let a = textureSample(history, linear_sample, uv + o15);
    let b = textureSample(history, linear_sample, uv - o15);
    new_color += vec4(a.rgb + b.rgb, 2.0) * weights[15];

    return vec4(new_color.rgb / new_color.w, original_color.a);
    // return original_color;
}