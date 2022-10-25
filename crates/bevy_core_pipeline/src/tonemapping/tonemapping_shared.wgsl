#define_import_path bevy_core_pipeline::tonemapping

// from https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve
fn aces_filmic(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((color * (a * color + b)) / (color * (c * color + d) + e));
}

// from https://64.github.io/tonemapping
// reinhard on RGB oversaturates colors
fn reinhard(color: vec3<f32>) -> vec3<f32> {
    return color / (1.0 + color);
}

fn reinhard_extended(color: vec3<f32>, max_white: f32) -> vec3<f32> {
    let numerator = color * (1.0 + (color / vec3<f32>(max_white * max_white)));
    return numerator / (1.0 + color);
}

// luminance coefficients from Rec. 709
// https://en.wikipedia.org/wiki/Rec._709
fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn reinhard_luminance(color: vec3<f32>) -> vec3<f32> {
    return color / (1.0 + luminance(color));
}

fn inverse_reinhard_luminance(color: vec3<f32>) -> vec3<f32> {
    return color / (1.0 - luminance(color));
}
