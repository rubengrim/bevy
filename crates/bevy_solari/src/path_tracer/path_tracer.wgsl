#import bevy_solari::scene_bindings
#import bevy_render::view
#import bevy_solari::utils

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var accumulation_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(2)
var output_texture: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(3)
var<storage, read_write> rays: array<RayDesc2>;
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

    let ray_hit = trace_ray(ray_origin, ray_direction, ray_t_min);
    if ray_hit.kind != RAY_QUERY_INTERSECTION_NONE {
        let ray_hit = map_ray_hit(ray_hit);

        ray_origin = ray_hit.world_position;
        ray_direction = sample_cosine_hemisphere(ray_hit.world_normal, &rng);
    }
    rays[pixel_index] = RayDesc2(vec4(ray_origin, bitcast<f32>(global_id.x)), vec4(ray_direction, bitcast<f32>(global_id.y)));
}
