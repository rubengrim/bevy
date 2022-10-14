struct CustomMaterial {
    color: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let texture_color = textureSample(base_color_texture, base_color_sampler, uv);

    // if (texture_color.a < 0.5) {
    //     discard;
    // }

    return vec4<f32>(material.color, 1.0) * texture_color;
}
