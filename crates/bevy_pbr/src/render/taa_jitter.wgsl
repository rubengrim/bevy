#define_import_path bevy_pbr::taa_jitter

fn halton_sequence(index: u32, base: u32) -> f32 {
    var f = 1.0;
    var r = 0.0;

    for (var i = index; i > 0u; i /= base) {
        f /= f32(base);
        r += f * f32(i % base);
    }

    return r;
}

fn taa_jitter(projection: mat4x4<f32>) -> mat4x4<f32> {
    let index = (globals.frame_count % 8u) + 1u;
    let halton = vec2<f32>(halton_sequence(index, 2u), halton_sequence(index, 3u));
    let jitter = (halton - 0.5) / view.viewport.zw;

    var new_projection = projection;
    new_projection[2][0] += jitter.x;
    new_projection[2][1] += jitter.y;
    return new_projection;
}
