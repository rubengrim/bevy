#import bevy_render::view

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var tlas: acceleration_structure;
@group(0) @binding(2)
var output_texture: texture_storage_2d<rgba16float, write>;

const RAY_QUERY_MASK: u32 = 0xFFu;
const RAY_QUERY_T_MIN: f32 = 0.001;
const RAY_QUERY_T_MAX: f32 = 10000.0;

@compute @workgroup_size(8, 8, 1)
fn solari_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_center = vec2<f32>(global_id.xy) + vec2(0.5);
    let pixel_uv = pixel_center / view.viewport.zw;
    let pixel_ndc = pixel_uv * 2.0 - 1.0;

    let ray_origin = view.world_position;
    let ray_target = view.inverse_projection * vec4(pixel_ndc, 1.0, 1.0);
    let ray_direction = (view.inverse_view * vec4(normalize(ray_target.xyz), 0.0)).xyz;

    var rq: ray_query;
    rayQueryInitialize(&rq, tlas, RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT, RAY_QUERY_MASK, RAY_QUERY_T_MIN, RAY_QUERY_T_MAX, ray_origin, ray_direction));
    rayQueryProceed(&rq);
    let ray_hit = rayQueryGetCommittedIntersection(&rq);

    var color = vec3(0.0);
    if (ray_hit.kind != RAY_QUERY_INTERSECTION_NONE) {
        color = vec3(ray_hit.barycentrics, 1.0 - ray_hit.barycentrics.x - intersection.barycentrics.y);
    };
    textureStore(output_texture, vec2<i32>(global_id.xy), vec4(color, 1.0));
}
