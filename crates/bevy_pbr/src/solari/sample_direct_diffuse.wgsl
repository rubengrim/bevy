#import bevy_render::view::View
#import bevy_render::globals::Globals
#import bevy_pbr::solari::bindings::{trace_ray, resolve_ray_hit, sample_light_sources, trace_light_source, RAY_T_MIN, RAY_T_MAX}
#import bevy_pbr::utils::{rand_f, rand_range_u}
#import bevy_core_pipeline::tonemapping::tonemapping_luminance

@group(2) @binding(0) var direct_diffuse: texture_storage_2d<rgba16float, read_write>;
@group(2) @binding(1) var view_output: texture_storage_2d<rgba16float, write>;
@group(2) @binding(2) var<uniform> view: View;
@group(2) @binding(3) var<uniform> globals: Globals;

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
    // This wouldn't compile, assuming it's not finished.
    // if any(global_id.xy >= vec2u(view.viewport.zw)) {
    //     return;
    // }

    // let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);
    // let frame_index = globals.frame_count * 5782582u;
    // var rng = pixel_index + frame_index;

    // let position = vec3(0.0); // TODO
    // let world_normal = vec3(0.0); // TODO

    // var reservoir = Reservoir(0u, 0u, 0.0, 0.0, 0u);
    // let light_count = arrayLength(&light_sources);
    // for (var i = 0u; i < 32u; i++) {
    //     let light_id = rand_range_u(light_count, &rng);
    //     let light = light_sources[light_id];

    //     let light_rng = rng;
    //     let sample = sample_light_sources(light_id, light_count, position, world_normal, &rng);
    //     let p_hat = tonemapping_luminance(sample.radiance);
    //     let light_weight = p_hat / sample.pdf;

    //     update_reservoir(&reservoir, light_id, light_rng, light_weight, &rng);
    // }

    // rng = reservoir.light_rng;
    // var radiance = trace_light_source(reservoir.light_id, light_count, position, world_normal, &rng);

    // let p_hat = tonemapping_luminance(radiance);
    // let w = reservoir.weight_sum / (p_hat * f32(reservoir.sample_count));
    // reservoir.light_weight = select(0.0, w, p_hat > 0.0);

    // radiance *= reservoir.light_weight;
}
