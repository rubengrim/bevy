#import bevy_pbr::prepass_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::taa_jitter

struct Vertex {
    @location(0) position: vec3<f32>,

#ifdef OUTPUT_NORMALS
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif // VERTEX_UVS
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif // VERTEX_TANGENTS
#endif // OUTPUT_NORMALS

#ifdef SKINNED
    @location(4) joint_indices: vec4<u32>,
    @location(5) joint_weights: vec4<f32>,
#endif // SKINNED
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
#ifdef OUTPUT_NORMALS
    @location(0) world_normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(1) uv: vec2<f32>,
#endif // VERTEX_UVS
#ifdef VERTEX_TANGENTS
    @location(2) world_tangent: vec4<f32>,
#endif // VERTEX_TANGENTS
#endif // OUTPUT_NORMALS

#ifdef OUTPUT_VELOCITIES
    @location(3) world_position: vec4<f32>,
    @location(4) previous_world_position: vec4<f32>,
#endif // OUTPUT_VELOCITIES
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var projection = view.projection;
#ifdef TEMPORAL_ANTI_ALIASING
    projection = taa_jitter(projection);
#endif

#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
#else // SKINNED
    var model = mesh.model;
#endif // SKINNED

#ifdef OUTPUT_VELOCITIES
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.previous_world_position = mesh_position_local_to_world(mesh.previous_model, vec4<f32>(vertex.position, 1.0));
#endif // OUTPUT_VELOCITIES

    out.clip_position = projection * view.inverse_view * out.world_position;

#ifdef OUTPUT_NORMALS
#ifdef SKINNED
    out.world_normal = skin_normals(model, vertex.normal);
#else // SKINNED
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
#endif // SKINNED

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif // VERTEX_UVS

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
#endif // VERTEX_TANGENTS
#endif // OUTPUT_NORMALS

    return out;
}

struct FragmentInput {
#ifdef OUTPUT_NORMALS
    @location(0) world_normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(1) uv: vec2<f32>,
#endif // VERTEX_UVS
#ifdef VERTEX_TANGENTS
    @location(2) world_tangent: vec4<f32>,
#endif // VERTEX_TANGENTS
#endif // OUTPUT_NORMALS

#ifdef OUTPUT_VELOCITIES
    // FIXME: Can we use @builtin(position)?
    @location(3) world_position: vec4<f32>,
    @location(4) previous_world_position: vec4<f32>,
#endif // OUTPUT_VELOCITIES
}

struct FragmentOutput {
    // FIXME: Need to vary locations based on enabled features
    @location(0) normal: vec4<f32>,
    @location(1) velocity: vec2<f32>,
}

fn clip_to_uv(clip: vec4<f32>) -> vec2<f32> {
    var uv = clip.xy / clip.w;
    uv = (uv + 1.0) * 0.5;
    uv.y = 1.0 - uv.y;
    return uv;
}

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;

#ifdef OUTPUT_NORMALS
    out.normal = vec4<f32>(in.world_normal * 0.5 + vec3<f32>(0.5), 1.0);
#endif

#ifdef OUTPUT_VELOCITIES
    let clip_position = view.view_proj * in.world_position;
    let previous_clip_position = previous_view_proj * in.previous_world_position;
    out.velocity = clip_to_uv(clip_position) - clip_to_uv(previous_clip_position);
#endif

    return out;
}
