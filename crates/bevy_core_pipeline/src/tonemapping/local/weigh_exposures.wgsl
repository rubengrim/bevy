@group(0) @binding(0) var luminances_image: texture_2d<f32>;
@group(0) @binding(1) var weights_image: texture_2d<f32>;
@group(0) @binding(2) var assembly_image: texture_storage_2d<r16float, write>;

@compute
@workgroup_size(8, 8, 1)
fn weigh_exposures(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coordinates = vec2<i32>(global_id.xy);
    var weights = textureLoad(weights_image, pixel_coordinates, 0).rgb;
    let luminances = textureLoad(luminances_image, pixel_coordinates, 0).rgb;

    weights /= dot(weights, vec3(1.0)) + 0.0001;
    let exposure = dot(luminances * weights, vec3(1.0));

    textureStore(assembly_image, pixel_coordinates, vec4(exposure, vec3(0.0)));
}
