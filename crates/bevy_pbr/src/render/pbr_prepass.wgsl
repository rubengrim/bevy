// FIXME: These imports are wrong, but they make it possible to import pbr functions without copying any code

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
// #import bevy_pbr::prepass_bindings

@group(0) @binding(1)
var<uniform> previous_view_proj: mat4x4<f32>;

#import bevy_pbr::pbr_bindings
#import bevy_pbr::utils
#import bevy_pbr::shadows
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_functions


struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,

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
};

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
    var output_color: vec4<f32> = material.base_color;

#ifdef VERTEX_UVS
    if ((material.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u) {
        output_color = output_color * textureSample(base_color_texture, base_color_sampler, in.uv);
    }
#endif

    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
#ifdef OUTPUT_NORMALS
    if ((material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u) {
        let normal = prepare_normal(
            material.flags,
            in.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
            in.world_tangent,
#endif
#endif
#ifdef VERTEX_UVS
            in.uv,
#endif
            in.is_front,
        );

        alpha_discard(material, output_color);
        out.normal = vec4<f32>(normal * 0.5 + vec3<f32>(0.5), 1.0);
    } else {
        alpha_discard(material, output_color);
        out.normal = vec4<f32>(in.world_normal * 0.5 + vec3<f32>(0.5), 1.0);
    }
#endif

#ifdef OUTPUT_VELOCITIES
    let clip_position = view.view_proj * in.world_position;
    let previous_clip_position = previous_view_proj * in.previous_world_position;
    out.velocity = clip_to_uv(clip_position) - clip_to_uv(previous_clip_position);
#endif

    return out;
}
