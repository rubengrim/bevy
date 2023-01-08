@group(0) @binding(0) var luminances_image: texture_2d<f32>;
@group(0) @binding(1) var luminances_previous_image: texture_2d<f32>;
@group(0) @binding(2) var weights_image: texture_2d<f32>;
@group(0) @binding(3) var assembly_previous_image: texture_2d<f32>;
@group(0) @binding(4) var assembly_image: texture_storage_2d<r16float, write>;
@group(0) @binding(5) var image_sampler: sampler;

@compute
@workgroup_size(8, 8, 1)
fn blend_laplacian(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coordinates = vec2<i32>(global_id.xy);
    let uv = vec2<f32>(pixel_coordinates) / vec2<f32>(textureDimensions(luminances_image));
    let luminances = textureLoad(luminances_image, pixel_coordinates, 0).xyz;
    let luminances_previous = textureSampleLevel(luminances_previous_image, image_sampler, uv, 0.0).xyz;
    var weights = textureLoad(weights_image, pixel_coordinates, 0).xyz;
    let assembly_previous = textureSampleLevel(assembly_previous_image, image_sampler, uv, 0.0).x;

    let laplacians = luminances - luminances_previous;
    weights /= dot(weights, vec3(1.0)) + 0.0001;
    let laplac = dot(laplacians * weights, vec3(1.0));
    let blended = assembly_previous + laplac;

    textureStore(assembly_image, pixel_coordinates, vec4(blended, vec3(0.0)));
}
