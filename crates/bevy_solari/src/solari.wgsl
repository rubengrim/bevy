#import bevy_solari::types
#import bevy_render::view

@group(0) @binding(0)
var tlas: acceleration_structure;
@group(0) @binding(1)
var<storage> mesh_materials: array<SolariMeshMaterial>;
@group(0) @binding(2)
var<storage> index_buffers: binding_array<SolariIndexBuffer>;
@group(0) @binding(3)
var<storage> vertex_buffers: binding_array<SolariVertexBuffer>;
@group(0) @binding(4)
var<storage> materials: array<SolariMaterial>;
@group(0) @binding(5)
var texture_maps: binding_array<texture_2d<f32>>;
@group(0) @binding(6)
var texture_sampler: sampler;

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var screen_probes: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(2)
var output_texture: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn solari_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_center = vec2<f32>(global_id.xy) + 0.5;
    let pixel_uv = pixel_center / view.viewport.zw;
    let pixel_ndc = (pixel_uv * 2.0) - 1.0;
    let primary_ray_target = view.inverse_view_proj * vec4(pixel_ndc.x, -pixel_ndc.y, 1.0, 1.0);

    var color = vec3(0.0);
    var ray_origin = view.world_position;
    var ray_direction = normalize((primary_ray_target.xyz / primary_ray_target.w) - ray_origin);

    let ray_hit = trace_ray(ray_origin, ray_direction);
    if (ray_hit.kind != RAY_QUERY_INTERSECTION_NONE) {
        let ray_hit = map_ray_hit(ray_hit);

        color += ray_hit.material.base_color;
    }

    textureStore(output_texture, global_id.xy, vec4(color, 1.0));
}
