#import bevy_render::view::View
#import bevy_pbr::solari::bindings::{trace_ray, resolve_ray_hit, sample_cosine_hemisphere}
#import bevy_pbr::utils::{rand_f, rand_vec2f}

@group(2) @binding(0) var accumulation_texture: texture_storage_2d<rgba32float, read_write>;
@group(2) @binding(1) var output_texture: texture_storage_2d<rgba16float, write>;
@group(2) @binding(2) var<uniform> view: View;

@compute @workgroup_size(8, 8, 1)
fn path_trace(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if any(global_id.xy >= vec2u(view.viewport.zw)) {
        return;
    }

    let old_color = textureLoad(accumulation_texture, global_id.xy);

    let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);
    let frame_index = u32(old_color.a) * 5782582u;
    var rng = pixel_index + frame_index;

    let pixel_center = vec2<f32>(global_id.xy) + 0.5;
    let jitter = rand_vec2f(&rng) - 0.5;
    let pixel_uv = (pixel_center + jitter) / view.viewport.zw;
    let pixel_ndc = (pixel_uv * 2.0) - 1.0;
    let primary_ray_target = view.inverse_view_proj * vec4(pixel_ndc.x, -pixel_ndc.y, 1.0, 1.0);
    var ray_origin = view.world_position;
    var ray_direction = normalize((primary_ray_target.xyz / primary_ray_target.w) - ray_origin);
    var ray_t_min = 0.0;

    var color = vec3(0.0);
    var throughput = vec3(1.0);
    loop {
        let ray_hit = trace_ray(ray_origin, ray_direction, ray_t_min, 10000.0);
        if ray_hit.kind != RAY_QUERY_INTERSECTION_NONE {
            let ray_hit = resolve_ray_hit(ray_hit);

            color += ray_hit.material.emissive * throughput;
            throughput *= ray_hit.material.base_color;

            let p = max(max(throughput.r, throughput.g), throughput.b);
            if rand_f(&rng) > p { break; }
            throughput *= 1.0 / p;

            ray_origin = ray_hit.world_position;
            ray_direction = sample_cosine_hemisphere(ray_hit.world_normal, &rng);
            ray_t_min = 0.001;
        } else { break; }
    }

    color *= view.exposure;

    let new_color = (color + old_color.a * old_color.rgb) / (old_color.a + 1.0);
    textureStore(accumulation_texture, global_id.xy, vec4(new_color, old_color.a + 1.0));
    textureStore(output_texture, global_id.xy, vec4(new_color, 1.0));
}