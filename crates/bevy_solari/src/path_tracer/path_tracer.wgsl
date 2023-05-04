#import bevy_solari::scene_types
#import bevy_solari::scene_bindings
#import bevy_solari::utils
#import bevy_render::view

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var accumulation_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(2)
var output_texture: texture_storage_2d<rgba16float, read_write>;
var<push_constant> previous_sample_count: f32;

@compute @workgroup_size(8, 8, 1)
fn path_trace(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);
    let frame_index = u32(previous_sample_count) * 5782582u;
    var rng = pixel_index + frame_index;

    let pixel_center = vec2<f32>(global_id.xy) + 0.5;
    let jitter = rand_vec2(&rng) - 0.5;
    let pixel_uv = (pixel_center + jitter) / view.viewport.zw;
    let pixel_ndc = (pixel_uv * 2.0) - 1.0;
    let primary_ray_target = view.inverse_view_proj * vec4(pixel_ndc.x, -pixel_ndc.y, 1.0, 1.0);
    var ray_origin = view.world_position;
    var ray_direction = normalize((primary_ray_target.xyz / primary_ray_target.w) - ray_origin);
    var ray_t_min = 0.0;

    // TODO: Next event estimation
    // TODO: Specular BRDF
    // TODO: BRDF energy conservation

    var color = vec3(0.0);
    var throughput = vec3(1.0);
    loop {
        let ray_hit = trace_ray(ray_origin, ray_direction, ray_t_min);
        if ray_hit.kind != RAY_QUERY_INTERSECTION_NONE {
            let ray_hit = map_ray_hit(ray_hit);

            color += ray_hit.material.emission * throughput;
            throughput *= ray_hit.material.base_color;

            let p = max(max(throughput.r, throughput.g), throughput.b);
            if rand_f(&rng) > p { break; }
            throughput *= 1.0 / p;

            ray_origin = ray_hit.world_position;
            ray_direction = sample_cosine_hemisphere(ray_hit.world_normal, &rng);
            ray_t_min = 0.001;
        } else { break; }
    }

    let old_color = textureLoad(accumulation_texture, global_id.xy).rgb;
    let new_color = vec4((color + previous_sample_count * old_color) / (previous_sample_count + 1.0), 1.0);

    textureStore(accumulation_texture, global_id.xy, new_color);
    textureStore(output_texture, global_id.xy, new_color);
}
