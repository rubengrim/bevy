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
        let mesh_material = mesh_materials[ray_hit.instance_custom_index];
        let index_buffer = &index_buffers[mesh_material.mesh_index].buffer;
        let vertex_buffer = &vertex_buffers[mesh_material.mesh_index].buffer;
        let material = materials[mesh_material.material_index];

        let indices_i = (ray_hit.primitive_index * 3u) + vec3(0u, 1u, 2u);
        let indices = vec3((*index_buffer)[indices_i.x], (*index_buffer)[indices_i.y], (*index_buffer)[indices_i.z]);
        let vertices = array<SolariVertex, 3>(unpack_vertex((*vertex_buffer)[indices.x]), unpack_vertex((*vertex_buffer)[indices.y]), unpack_vertex((*vertex_buffer)[indices.z]));
        let barycentrics = vec3(1.0 - ray_hit.barycentrics.x - ray_hit.barycentrics.y, ray_hit.barycentrics);
        let uv = mat3x2(vertices[0].uv, vertices[1].uv, vertices[2].uv) * barycentrics;

        color = material.base_color.rgb;
        if material.base_color_map_index != TEXTURE_MAP_NONE {
            color *= textureSampleLevel(texture_maps[material.base_color_map_index], texture_sampler, uv, 0.0).rgb;
        }
    };

    textureStore(output_texture, global_id.xy, vec4(color, 1.0));
}
