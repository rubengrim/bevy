#define_import_path bevy_pbr::taa_jitter

fn taa_jitter(projection: mat4x4<f32>) -> mat4x4<f32> {
    // TODO: When naga implements it, this can be a module-level const
    // Halton sequence (2, 3), -0.5
    var halton_sequence = array(
        vec2(0.0, -0.16666666),
        vec2(-0.25, 0.16666669),
        vec2(0.25, -0.3888889),
        vec2(-0.375, -0.055555552),
        vec2(0.125, 0.2777778),
        vec2(-0.125, -0.2777778),
        vec2(0.375, 0.055555582),
        vec2(-0.4375, 0.3888889),
        vec2(0.0625, -0.46296296),
        vec2(-0.1875, -0.12962961),
        vec2(0.3125, 0.2037037),
        vec2(-0.3125, -0.35185185),
    );
    let jitter = halton_sequence[globals.frame_count % 12u] / view.viewport.zw;

    var new_projection = projection;
    new_projection[2][0] += jitter.x;
    new_projection[2][1] += jitter.y;
    return new_projection;
}
