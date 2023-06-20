#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils

@compute @workgroup_size(8, 8, 1)
fn sample_direct_diffuse(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if any(global_id.xy >= screen_size) { return; }

    let light_count = arrayLength(&emissive_object_indices);
    if light_count == 0u {
        textureStore(view_target, global_id.xy, vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }

    let pixel_index = global_id.x + global_id.y * screen_size.x;
    let frame_index = globals.frame_count * 5782582u;
    var rng = pixel_index + frame_index;

    let g_buffer_pixel = textureLoad(g_buffer, global_id.xy);
    let pixel_depth = decode_g_buffer_depth(g_buffer_pixel);
    if pixel_depth < 0.0 {
        textureStore(view_target, global_id.xy, vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }
    let pixel_id = vec2<f32>(global_id.xy) + 0.5;
    let pixel_world_position = depth_to_world_position(pixel_depth, pixel_id / view.viewport.zw);
    let pixel_world_normal = decode_g_buffer_world_normal(g_buffer_pixel);
    let pixel_material = decode_m_buffer(textureLoad(m_buffer, global_id.xy));
    let pixel_brdf = pixel_material.base_color / PI;

    var reservoir_unshadowed_light_contribution = vec3(0.0);
    var reservoir_light_position = vec3(0.0);
    var reservoir_target_pdf = 0.0;
    var reservoir_weight = 0.0;
    for (var m = 0u; m < 32u; m++) {
        // TODO: Abstract into function shared with sample_direct_lighting()
        let light_i = rand_range_u(light_count, &rng);
        let triangle_count = emissive_object_triangle_counts[light_i];
        let triangle_i = rand_range_u(triangle_count, &rng);
        let light_object_i = emissive_object_indices[light_i];
        let light_mm_indices = mesh_material_indices[light_object_i];
        let light_transform = transforms[light_object_i];
        let mesh_index = light_mm_indices >> 16u;
        let material_index = light_mm_indices & 0xFFFFu;
        let index_buffer = &index_buffers[mesh_index].buffer;
        let vertex_buffer = &vertex_buffers[mesh_index].buffer;
        let material = materials[material_index];
        let indices_i = (triangle_i * 3u) + vec3(0u, 1u, 2u);
        let indices = vec3((*index_buffer)[indices_i.x], (*index_buffer)[indices_i.y], (*index_buffer)[indices_i.z]);
        let vertices = array<SolariVertex, 3>(unpack_vertex((*vertex_buffer)[indices.x]), unpack_vertex((*vertex_buffer)[indices.y]), unpack_vertex((*vertex_buffer)[indices.z]));
        var r = rand_vec2(&rng);
        if r.x + r.y > 1.0 { r = 1.0 - r; }
        let barycentrics = vec3(r, 1.0 - r.x - r.y);
        let local_position = mat3x3(vertices[0].local_position, vertices[1].local_position, vertices[2].local_position) * barycentrics;
        let world_position = (light_transform * vec4(local_position, 1.0)).xyz;
        let light_distance = distance(pixel_world_position, world_position);
        let ray_direction = (world_position - pixel_world_position) / light_distance;
        let local_normal = mat3x3(vertices[0].local_normal, vertices[1].local_normal, vertices[2].local_normal) * barycentrics;
        var world_normal = normalize(mat3x3(light_transform[0].xyz, light_transform[1].xyz, light_transform[2].xyz) * local_normal);
        if material.normal_map_index != TEXTURE_MAP_NONE {
            let uv = mat3x2(vertices[0].uv, vertices[1].uv, vertices[2].uv) * barycentrics;
            let local_tangent = mat3x3(vertices[0].local_tangent.xyz, vertices[1].local_tangent.xyz, vertices[2].local_tangent.xyz) * barycentrics;
            let world_tangent = normalize(mat3x3(light_transform[0].xyz, light_transform[1].xyz, light_transform[2].xyz) * local_tangent);
            let N = world_normal;
            let T = world_tangent;
            let B = vertices[0].local_tangent.w * cross(N, T);
            let Nt = textureSampleLevel(texture_maps[material.normal_map_index], texture_sampler, uv, 0.0).rgb;
            world_normal = normalize(Nt.x * T + Nt.y * B + Nt.z * N);
        }
        let cos_theta_origin = saturate(dot(ray_direction, pixel_world_normal));
        let cos_theta_light = saturate(dot(-ray_direction, world_normal));
        let light_distance_squared = light_distance * light_distance;
        let light = material.emission * cos_theta_origin * (cos_theta_light / light_distance_squared);
        let triangle_edge0 = vertices[0].local_position - vertices[1].local_position;
        let triangle_edge1 = vertices[0].local_position - vertices[2].local_position;
        let triangle_area = length(cross(triangle_edge0, triangle_edge1)) / 2.0;

        let sample_pdf = 1.0 / (f32(light_count * triangle_count) * triangle_area);
        let target_pdf = dot(pixel_brdf * light, vec3(0.2126729, 0.7151522, 0.0721750));

        let sample_weight = target_pdf / sample_pdf;
        reservoir_weight += sample_weight;
        if rand_f(&rng) < sample_weight / reservoir_weight {
            reservoir_unshadowed_light_contribution = light;
            reservoir_light_position = world_position;
            reservoir_target_pdf = target_pdf;
        }
    }

    var direct_light = (reservoir_unshadowed_light_contribution * reservoir_weight) / (reservoir_target_pdf * 32.0);
    direct_light = max(vec3(0.0), direct_light);

    // TODO: Abstract into function shared with sample_direct_lighting()
    let light_distance = distance(pixel_world_position, reservoir_light_position);
    let ray_flags = RAY_FLAG_TERMINATE_ON_FIRST_HIT;
    let ray_cull_mask = 0xFFu;
    let ray_t_min = 0.01;
    let ray_t_max = light_distance - 0.01;
    let ray_direction = (reservoir_light_position - pixel_world_position) / light_distance;
    let ray = RayDesc(ray_flags, ray_cull_mask, ray_t_min, ray_t_max, pixel_world_position, ray_direction);
    var rq: ray_query;
    rayQueryInitialize(&rq, tlas, ray);
    rayQueryProceed(&rq);
    let ray_hit = rayQueryGetCommittedIntersection(&rq);
    if ray_hit.kind != RAY_QUERY_INTERSECTION_NONE {
        direct_light = vec3(0.0);
    }

    textureStore(direct_diffuse, global_id.xy, vec4(direct_light, 1.0));
}
