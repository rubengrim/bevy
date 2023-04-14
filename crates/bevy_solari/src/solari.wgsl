#import bevy_render::view
#import bevy_solari::material

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var tlas: acceleration_structure;
@group(0) @binding(2)
var<storage> materials: array<SolariMaterial>;
@group(0) @binding(3)
var output_texture: texture_storage_2d<rgba16float, write>;

fn trace_ray(pixel_index: vec2<u32>) -> RayIntersection {
    let pixel_center = vec2<f32>(pixel_index) + 0.5;
    let pixel_uv = pixel_center / view.viewport.zw;
    let pixel_ndc = (pixel_uv * 2.0) - 1.0;

    let ray_origin = view.world_position;
    let ray_target = view.inverse_view_proj * vec4(pixel_ndc.x, -pixel_ndc.y, 1.0, 1.0);
    let ray_direction = normalize((ray_target.xyz / ray_target.w) - ray_origin);

    let ray_flags = RAY_FLAG_NONE;
    let ray_cull_mask = 0xFFu;
    let ray_t_min = 0.001;
    let ray_t_max = 10000.0;
    let ray = RayDesc(ray_flags, ray_cull_mask, ray_t_min, ray_t_max, ray_origin, ray_direction);

    var rq: ray_query;
    rayQueryInitialize(&rq, tlas, ray);
    rayQueryProceed(&rq);
    return rayQueryGetCommittedIntersection(&rq);
}

@compute @workgroup_size(8, 8, 1)
fn solari_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var color = vec3(0.0);

    let ray_hit = trace_ray(global_id.xy);
    if (ray_hit.kind != RAY_QUERY_INTERSECTION_NONE) {
        color = materials[ray_hit.instance_custom_index].base_color.rgb;
    };

    textureStore(output_texture, global_id.xy, vec4(color, 1.0));
}
