#define_import_path bevy_pbr::taa_jitter

fn taa_jitter(projection: mat4x4<f32>) -> mat4x4<f32> {
    var jitter: vec2<f32> = vec2<f32>(0.0, 0.0);
    // TODO

    var new_projection = projection;
    new_projection[2][0] += jitter.x;
    new_projection[2][1] += jitter.y;
    return new_projection;
}
