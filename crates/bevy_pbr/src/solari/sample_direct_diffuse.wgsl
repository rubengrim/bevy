#import bevy_render::view::View
#import bevy_pbr::solari::bindings::{trace_ray, resolve_ray_hit, sample_cosine_hemisphere, RAY_T_MIN, RAY_T_MAX}
#import bevy_pbr::utils::rand_f
// #import bevy_core_pipeline::tonemapping::tonemapping_luminance

@group(2) @binding(0) var direct_diffuse: texture_storage_2d<rgba16float, read_write>;
@group(2) @binding(1) var view_output: texture_storage_2d<rgba16float, write>;
@group(2) @binding(2) var<uniform> view: View;

struct Reservoir {
    light_id: u32,
    light_rng: u32,
    light_weight: f32,
    weight_sum: f32,
    sample_count: u32
}

fn update_reservoir(reservoir: ptr<function, Reservoir>, light_id: u32, light_rng: u32, light_weight: f32, rng: ptr<function, u32>) {
    (*reservoir).weight_sum += light_weight;
    (*reservoir).sample_count += 1u;
    if rand_f(rng) < light_weight / (*reservoir).weight_sum {
        (*reservoir).light_id = light_id;
        (*reservoir).light_rng = light_rng;
    }
}

@compute @workgroup_size(8, 8, 1)
fn sample_direct_diffuse(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if any(global_id.xy >= vec2u(view.viewport.zw)) {
        return;
    }

    var reservoir = Resevoir(0u, 0u, 0.0, 0.0, 0u);
    let light_count = arrayLength(&light_sources);
    for (var i = 0u; i < 32u; i++) {
        let light_id = rand_range_u(light_count, state);
        let light = light_sources[light_id];
    }
}
