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
    var uv = vec2(-1.0);
    var motion_vector = vec2(0.0);


    let primary_ray_direction = pixel_to_ray_direction((vec2<f32>(global_id.xy) + 0.5) / view.viewport.zw);
    let ray_hit = trace_ray(view.world_position, primary_ray_direction, 0.0);

    if ray_hit.kind != RAY_QUERY_INTERSECTION_NONE {
        let mm_indices = mesh_material_indices[ray_hit.instance_custom_index];
        let mesh_index = mm_indices >> 16u;
        material_index = mm_indices & 0xFFFFu;
        let index_buffer = &index_buffers[mesh_index].buffer;
        let vertex_buffer = &vertex_buffers[mesh_index].buffer;
        let material = materials[material_index];
        let indices_i = (ray_hit.primitive_index * 3u) + vec3(0u, 1u, 2u);
        let indices = vec3((*index_buffer)[indices_i.x], (*index_buffer)[indices_i.y], (*index_buffer)[indices_i.z]);
        let vertices = array<SolariVertex, 3>(unpack_vertex((*vertex_buffer)[indices.x]), unpack_vertex((*vertex_buffer)[indices.y]), unpack_vertex((*vertex_buffer)[indices.z]));
        let barycentrics = vec3(1.0 - ray_hit.barycentrics.x - ray_hit.barycentrics.y, ray_hit.barycentrics);
        let local_position = mat3x3(vertices[0].local_position, vertices[1].local_position, vertices[2].local_position) * barycentrics;
        let world_position = (ray_hit.object_to_world * vec4(local_position, 1.0)).xyz;
        uv = mat3x2(vertices[0].uv, vertices[1].uv, vertices[2].uv) * barycentrics;
        let transform = transforms[ray_hit.instance_custom_index];
        let local_normal = mat3x3(vertices[0].local_normal, vertices[1].local_normal, vertices[2].local_normal) * barycentrics;
        world_normal = normalize(mat3x3(transform[0].xyz, transform[1].xyz, transform[2].xyz) * local_normal);
        if material.normal_map_index != TEXTURE_MAP_NONE {
            let local_tangent = mat3x3(vertices[0].local_tangent.xyz, vertices[1].local_tangent.xyz, vertices[2].local_tangent.xyz) * barycentrics;
            let world_tangent = normalize(mat3x3(transform[0].xyz, transform[1].xyz, transform[2].xyz) * local_tangent);
            let N = world_normal;
            let T = world_tangent;
            let B = vertices[0].local_tangent.w * cross(N, T);
            let Nt = textureSampleLevel(texture_maps[material.normal_map_index], texture_sampler, uv, 0.0).rgb;
            world_normal = normalize(Nt.x * T + Nt.y * B + Nt.z * N);
        }

        ray_distance = ray_hit.t;

        var current_clip_position = ((vec2<f32>(global_id.xy) + 0.5) / view.viewport.zw) * 2.0 - 1.0;
        current_clip_position.y *= -1.0;
        // TODO: I think there's still jitter here due to the use of `barycentrics` coming from the jittered ray
        let current_local_position = mat3x3(vertices[0].local_position, vertices[1].local_position, vertices[2].local_position) * barycentrics;
        let previous_world_position = previous_transforms[ray_hit.instance_custom_index] * vec4(current_local_position, 1.0);
        let previous_clip_position_t = previous_view_proj * previous_world_position;
        let previous_clip_position = previous_clip_position_t.xy / previous_clip_position_t.w;
        motion_vector = (current_clip_position - previous_clip_position) * vec2(0.5, -0.5);
    }

    textureStore(g_buffer, global_id.xy, encode_g_buffer(ray_distance, world_normal));
    textureStore(m_buffer, global_id.xy, encode_m_buffer(material_index, uv));
    textureStore(t_buffer, global_id.xy, vec4(motion_vector, 0.0, 0.0));
}
