#import bevy_pbr::meshlet_bindings::{
    meshlet_previous_thread_ids,
    meshlet_previous_occlusion,
    meshlet_occlusion,
    meshlet_thread_meshlet_ids,
    meshlets,
    draw_command_buffer,
    draw_index_buffer,
    meshlet_thread_instance_ids,
    meshlet_instance_uniforms,
    meshlet_bounding_spheres,
    view,
}
#ifdef MESHLET_SECOND_CULLING_PASS
#import bevy_pbr::meshlet_bindings::depth_pyramid
#endif
#import bevy_render::maths::affine_to_square

/// Culls individual meshlets in two passes (two pass occlusion culling), and creates draw indirect commands for each surviving meshlet.
/// 1. The first pass is only frustum culling, and only the meshlets that were visible last frame get rendered.
/// 2. The second pass then performs both frustum and occlusion culling (using the depth buffer generated from the first pass) for all meshlets,
///    and stores whether each meshlet was culled for the first pass in the next frame.

@compute
@workgroup_size(128, 1, 1)
fn cull_meshlets(@builtin(global_invocation_id) thread_id: vec3<u32>) {
    // Fetch the instanced meshlet data
    if thread_id.x >= arrayLength(&meshlet_thread_meshlet_ids) { return; }
    let meshlet_id = meshlet_thread_meshlet_ids[thread_id.x];
    let bounding_sphere = meshlet_bounding_spheres[meshlet_id];
    let instance_id = meshlet_thread_instance_ids[thread_id.x];
    let instance_uniform = meshlet_instance_uniforms[instance_id];
    let model = affine_to_square(instance_uniform.model);
    let model_scale = max(length(model[0]), max(length(model[1]), length(model[2])));
    let bounding_sphere_center = model * vec4(bounding_sphere.center, 1.0);
    let bounding_sphere_radius = model_scale * bounding_sphere.radius;

#ifdef MESHLET_SECOND_CULLING_PASS
    var meshlet_visible = true;
#else
    // In the first culling pass, cull all meshlets that were not visible last frame
    let previous_thread_id = meshlet_previous_thread_ids[thread_id.x];
    var meshlet_visible = bool(meshlet_previous_occlusion[previous_thread_id]);
#endif

    // Frustum culling
    // TODO: Faster method from https://vkguide.dev/docs/gpudriven/compute_culling/#frustum-culling-function
    for (var i = 0u; i < 6u; i++) {
        if !meshlet_visible { break; }
        meshlet_visible &= dot(view.frustum[i], bounding_sphere_center) > -bounding_sphere_radius;
    }

#ifdef MESHLET_SECOND_CULLING_PASS
    // In the second culling pass, cull against the depth pyramid generated from the first pass
    var aabb: vec4<f32>;
    let bounding_sphere_center_view_space = (view.inverse_view * vec4(bounding_sphere_center.xyz, 1.0)).xyz;
    if meshlet_visible && try_project_sphere(bounding_sphere_center_view_space, bounding_sphere_radius, &aabb) {
        let depth_pyramid_size = vec2<f32>(textureDimensions(depth_pyramid));
        let width = (aabb.z - aabb.x) * depth_pyramid_size.x;
        let height = (aabb.w - aabb.y) * depth_pyramid_size.y;
        let depth_level = i32(ceil(log2(max(width, height)))); // TODO: Naga dosen't like this being a u32
        let aabb_top_left = vec2<u32>(aabb.xy * depth_pyramid_size);

        let depth_quad_a = textureLoad(depth_pyramid, aabb_top_left, depth_level).x;
        let depth_quad_b = textureLoad(depth_pyramid, aabb_top_left + vec2(1u, 0u), depth_level).x;
        let depth_quad_c = textureLoad(depth_pyramid, aabb_top_left + vec2(0u, 1u), depth_level).x;
        let depth_quad_d = textureLoad(depth_pyramid, aabb_top_left + vec2(1u, 1u), depth_level).x;
        let occluder_depth = min(min(depth_quad_a, depth_quad_b), min(depth_quad_c, depth_quad_d));

        let sphere_depth = -view.projection[3][2] / (bounding_sphere_center_view_space.z + bounding_sphere_radius);
        meshlet_visible &= sphere_depth >= occluder_depth;
    }
#endif

    // If the meshlet is visible, atomically append its index buffer (packed together with the meshlet ID) to
    // the index buffer for the rasterization pass to use
    if meshlet_visible {
        let meshlet = meshlets[meshlet_id];
        let draw_index_buffer_start = atomicAdd(&draw_command_buffer.vertex_count, meshlet.index_count);
        let packed_thread_id = thread_id.x << 8u;
        for (var index_id = 0u; index_id < meshlet.index_count; index_id++) {
            draw_index_buffer[draw_index_buffer_start + index_id] = packed_thread_id | index_id;
        }
    }

#ifdef MESHLET_SECOND_CULLING_PASS
    // In the second culling pass, write out the visible meshlets for the first culling pass of the next frame
    meshlet_occlusion[thread_id.x] = u32(meshlet_visible);
#endif
}

// https://zeux.io/2023/01/12/approximate-projected-bounds
fn try_project_sphere(cp: vec3<f32>, r: f32, aabb_out: ptr<function, vec4<f32>>) -> bool {
    let c = vec3(cp.xy, -cp.z);

    if c.z < r + view.projection[3][2] {
        return false;
    }

    let cr = c * r;
    let czr2 = c.z * c.z - r * r;

    let vx = sqrt(c.x * c.x + czr2);
    let min_x = (vx * c.x - cr.z) / (vx * c.z + cr.x);
    let max_x = (vx * c.x + cr.z) / (vx * c.z - cr.x);

    let vy = sqrt(c.y * c.y + czr2);
    let min_y = (vy * c.y - cr.z) / (vy * c.z + cr.y);
    let max_y = (vy * c.y + cr.z) / (vy * c.z - cr.y);

    let p00 = view.projection[0][0];
    let p11 = view.projection[1][1];

    var aabb = vec4(min_x * p00, min_y * p11, max_x * p00, max_y * p11);
    aabb = aabb.xwzy * vec4(0.5, -0.5, 0.5, -0.5) + vec4(0.5);

    *aabb_out = aabb;
    return true;
}
