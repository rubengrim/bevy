#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils

@compute @workgroup_size(8, 8, 1)
fn gmt_buffer(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if global_id.x >= screen_size.x || global_id.y >= screen_size.y {
        return;
    }

    var ray_distance = -1.0;
    var world_normal = vec3(-1.0);
    var material_index = 999999u;
    var texture_coordinates = vec2(-1.0);
    var motion_vector = vec2(0.0);

    let pixel_index = global_id.x + global_id.y * screen_size.x;
    let frame_index = globals.frame_count * 5782582u;
    var rng = pixel_index + frame_index;

    let primary_ray_direction = pixel_to_ray_direction((vec2<f32>(global_id.xy) + rand_vec2(&rng)) / view.viewport.zw);
    let ray_hit = trace_ray(view.world_position, primary_ray_direction, 0.0);

    if ray_hit.kind != RAY_QUERY_INTERSECTION_NONE {
        let mesh_material = mesh_materials[ray_hit.instance_custom_index];
        let index_buffer = &index_buffers[mesh_material.mesh_index].buffer;
        let vertex_buffer = &vertex_buffers[mesh_material.mesh_index].buffer;
        let indices_i = (ray_hit.primitive_index * 3u) + vec3(0u, 1u, 2u);
        let indices = vec3((*index_buffer)[indices_i.x], (*index_buffer)[indices_i.y], (*index_buffer)[indices_i.z]);
        let vertices = array<SolariVertex, 3>(unpack_vertex((*vertex_buffer)[indices.x]), unpack_vertex((*vertex_buffer)[indices.y]), unpack_vertex((*vertex_buffer)[indices.z]));
        let barycentrics = vec3(1.0 - ray_hit.barycentrics.x - ray_hit.barycentrics.y, ray_hit.barycentrics);
        let local_normal = mat3x3(vertices[0].local_normal, vertices[1].local_normal, vertices[2].local_normal) * barycentrics;

        ray_distance = ray_hit.t;
        world_normal = normalize((local_normal * ray_hit.object_to_world).xyz);
        material_index = mesh_material.material_index;
        texture_coordinates = mat3x2(vertices[0].uv, vertices[1].uv, vertices[2].uv) * barycentrics;

        let current_clip_position = ((vec2<f32>(global_id.xy) + 0.5) / view.viewport.zw) * 2.0 - 1.0;
        // TODO: I think there's still jitter here due to the use of `barycentrics` coming from the jittered ray
        let current_local_position = mat3x3(vertices[0].local_position, vertices[1].local_position, vertices[2].local_position) * barycentrics;
        let previous_world_position = previous_transforms[ray_hit.instance_custom_index] * vec4(current_local_position, 1.0);
        let previous_clip_position_t = previous_view_proj * previous_world_position;
        let previous_clip_position = previous_clip_position_t.xy / previous_clip_position_t.w;
        motion_vector = (current_clip_position - previous_clip_position) * vec2(0.5, -0.5);
    }

    textureStore(g_buffer, global_id.xy, encode_g_buffer(ray_distance, world_normal));
    textureStore(m_buffer, global_id.xy, encode_m_buffer(material_index, texture_coordinates));
    textureStore(t_buffer, global_id.xy, vec4(motion_vector, 0.0, 0.0));
}
