#import bevy_core_pipeline::tonemapping

@group(0) @binding(0) var hdr_image: texture_2d<f32>;
@group(0) @binding(1) var luminances_image: texture_storage_2d<rgba16float, write>;

let EXPOSURE = 0.7;
let HIGHLIGHTS = 2.0;
let SHADOWS = 1.5;

fn luminance(hdr_color: vec3<f32>) -> f32 {
    let c = aces_filmic(hdr_color);
    let c = saturate(c);
    let l = dot(c, vec3(0.1, 0.7, 0.2));
    return sqrt(l);
}

@compute
@workgroup_size(8, 8, 1)
fn compute_luminances(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coordinates = vec2<i32>(global_id.xy);
    let highlights = pow(2.0, -HIGHLIGHTS);
    let shadows = pow(2.0, SHADOWS);

    let hdr_color = textureLoad(hdr_image, pixel_coordinates, 0).rgb * EXPOSURE;

    let highlights = luminance(hdr_color * highlights);
    let midtones = luminance(hdr_color);
    let shadows = luminance(hdr_color * shadows);

    textureStore(luminances_image, pixel_coordinates, vec4(highlights, midtones, shadows, 1.0));
}
