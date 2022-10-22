#define_import_path bevy_pbr::taa_jitter

fn taa_jitter(projection: mat4x4<f32>) -> mat4x4<f32> {
    var halton_sequence = array(
        vec2(0.5, 0.33333334),
        vec2(0.25, 0.6666667),
        vec2(0.75, 0.11111111),
        vec2(0.125, 0.44444445),
        vec2(0.625, 0.7777778),
        vec2(0.375, 0.22222222),
        vec2(0.875, 0.5555556),
        vec2(0.0625, 0.8888889),
        vec2(0.5625, 0.037037037),
        vec2(0.3125, 0.3703704),
        vec2(0.8125, 0.7037037),
        vec2(0.1875, 0.14814815),
    );
    let jitter = halton_sequence[globals.frame_count % 12u] / view.viewport.zw;

    var new_projection = projection;
    new_projection[2][0] += jitter.x;
    new_projection[2][1] += jitter.y;
    return new_projection;
}
