@group(0) @binding(0) var luminances_image_mip0: texture_2d<f32>;
@group(0) @binding(1) var luminances_image_mip1: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var luminances_image_mip2: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var luminances_image_mip3: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4) var luminances_image_mip4: texture_storage_2d<rgba16float, write>;
@group(0) @binding(5) var luminances_image_mip5: texture_storage_2d<rgba16float, write>;
@group(0) @binding(6) var weights_image_mip0: texture_storage_2d<rgba16float, write>;
@group(0) @binding(7) var weights_image_mip1: texture_storage_2d<rgba16float, write>;
@group(0) @binding(8) var weights_image_mip2: texture_storage_2d<rgba16float, write>;
@group(0) @binding(9) var weights_image_mip3: texture_storage_2d<rgba16float, write>;
@group(0) @binding(10) var weights_image_mip4: texture_storage_2d<rgba16float, write>;
@group(0) @binding(11) var weights_image_mip5: texture_storage_2d<rgba16float, write>;

let SIGMA = 5.0;

fn compute_weights_mip0(luminances: vec3<f32>, pixel_coordinates: vec2<i32>) -> vec3<f32> {
    let sigma_squared = SIGMA * SIGMA;

    let diff = luminances - vec3(0.5);
    var weights = exp(-0.5 * diff * diff * sigma_squared);
    weights /= dot(weights, vec3(1.0)) + 0.00001;

    textureStore(weights_image_mip0, pixel_coordinates, vec4(weights, 1.0));
    return weights;
}

var<workgroup> previous_mip_luminances: array<array<vec3<f32>, 8>, 8>;
var<workgroup> previous_mip_weights: array<array<vec3<f32>, 8>, 8>;

@compute
@workgroup_size(8, 8, 1)
fn compute_weights(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let base_coordinates = vec2<i32>(global_id.xy);
    let pixel_coordinates0 = base_coordinates * 2i;
    let pixel_coordinates1 = pixel_coordinates0 + vec2<i32>(1i, 0i);
    let pixel_coordinates2 = pixel_coordinates0 + vec2<i32>(0i, 1i);
    let pixel_coordinates3 = pixel_coordinates0 + vec2<i32>(1i, 1i);

    let luminances0 = textureLoad(luminances_image_mip0, pixel_coordinates0, 0).xyz;
    let luminances1 = textureLoad(luminances_image_mip0, pixel_coordinates1, 0).xyz;
    let luminances2 = textureLoad(luminances_image_mip0, pixel_coordinates2, 0).xyz;
    let luminances3 = textureLoad(luminances_image_mip0, pixel_coordinates3, 0).xyz;
    let luminances = (luminances0 + luminances1 + luminances2 + luminances3) / 4.0;

    // Mip 0
    let weight0 = compute_weights_mip0(luminances0, pixel_coordinates0);
    let weight1 = compute_weights_mip0(luminances0, pixel_coordinates1);
    let weight2 = compute_weights_mip0(luminances0, pixel_coordinates2);
    let weight3 = compute_weights_mip0(luminances0, pixel_coordinates3);
    let weight = (weight0 + weight1 + weight2 + weight3) / 4.0;

    // Mip 1
    textureStore(luminances_image_mip1, base_coordinates, vec4(luminances, 1.0));
    textureStore(weights_image_mip1, base_coordinates, vec4(weight, 1.0));
    previous_mip_luminances[local_id.x][local_id.y] = luminances;
    previous_mip_weights[local_id.x][local_id.y] = weight;

    workgroupBarrier();

    // Mip 2
    if all(local_id.xy % vec2<u32>(2u) == vec2<u32>(0u)) {
        let luminances0 = previous_mip_luminances[local_id.x + 0u][local_id.y + 0u];
        let luminances1 = previous_mip_luminances[local_id.x + 1u][local_id.y + 0u];
        let luminances2 = previous_mip_luminances[local_id.x + 0u][local_id.y + 1u];
        let luminances3 = previous_mip_luminances[local_id.x + 1u][local_id.y + 1u];
        let luminances = (luminances0 + luminances1 + luminances2 + luminances3) / 4.0;

        let weight0 = previous_mip_weights[local_id.x + 0u][local_id.y + 0u];
        let weight1 = previous_mip_weights[local_id.x + 1u][local_id.y + 0u];
        let weight2 = previous_mip_weights[local_id.x + 0u][local_id.y + 1u];
        let weight3 = previous_mip_weights[local_id.x + 1u][local_id.y + 1u];
        let weight = (weight0 + weight1 + weight2 + weight3) / 4.0;

        textureStore(luminances_image_mip2, base_coordinates / 2i, vec4(luminances, 1.0));
        textureStore(weights_image_mip2, base_coordinates / 2i, vec4(weight, 1.0));
        previous_mip_luminances[local_id.x][local_id.y] = luminances;
        previous_mip_weights[local_id.x][local_id.y] = weight;
    }

    workgroupBarrier();

    // Mip 3
    if all(local_id.xy % vec2<u32>(4u) == vec2<u32>(0u)) {
        let luminances0 = previous_mip_luminances[local_id.x + 0u][local_id.y + 0u];
        let luminances1 = previous_mip_luminances[local_id.x + 2u][local_id.y + 0u];
        let luminances2 = previous_mip_luminances[local_id.x + 0u][local_id.y + 2u];
        let luminances3 = previous_mip_luminances[local_id.x + 2u][local_id.y + 2u];
        let luminances = (luminances0 + luminances1 + luminances2 + luminances3) / 4.0;

        let weight0 = previous_mip_weights[local_id.x + 0u][local_id.y + 0u];
        let weight1 = previous_mip_weights[local_id.x + 2u][local_id.y + 0u];
        let weight2 = previous_mip_weights[local_id.x + 0u][local_id.y + 2u];
        let weight3 = previous_mip_weights[local_id.x + 2u][local_id.y + 2u];
        let weight = (weight0 + weight1 + weight2 + weight3) / 4.0;

        textureStore(luminances_image_mip3, base_coordinates / 4i, vec4(luminances, 1.0));
        textureStore(weights_image_mip3, base_coordinates / 4i, vec4(weight, 1.0));
        previous_mip_luminances[local_id.x][local_id.y] = luminances;
        previous_mip_weights[local_id.x][local_id.y] = weight;
    }

    workgroupBarrier();

    // Mip 4
    if all(local_id.xy % vec2<u32>(8u) == vec2<u32>(0u)) {
        let luminances0 = previous_mip_luminances[local_id.x + 0u][local_id.y + 0u];
        let luminances1 = previous_mip_luminances[local_id.x + 4u][local_id.y + 0u];
        let luminances2 = previous_mip_luminances[local_id.x + 0u][local_id.y + 4u];
        let luminances3 = previous_mip_luminances[local_id.x + 4u][local_id.y + 4u];
        let luminances = (luminances0 + luminances1 + luminances2 + luminances3) / 4.0;

        let weight0 = previous_mip_weights[local_id.x + 0u][local_id.y + 0u];
        let weight1 = previous_mip_weights[local_id.x + 4u][local_id.y + 0u];
        let weight2 = previous_mip_weights[local_id.x + 0u][local_id.y + 4u];
        let weight3 = previous_mip_weights[local_id.x + 4u][local_id.y + 4u];
        let weight = (weight0 + weight1 + weight2 + weight3) / 4.0;

        textureStore(luminances_image_mip4, base_coordinates / 8i, vec4(luminances, 1.0));
        textureStore(weights_image_mip4, base_coordinates / 8i, vec4(weight, 1.0));
        previous_mip_luminances[local_id.x][local_id.y] = luminances;
        previous_mip_weights[local_id.x][local_id.y] = weight;
    }

    workgroupBarrier();

    // Mip 5
    // TODO: This is probably wrong, still 1x1, but need to adjust workgroups maybe? idk
    if all(local_id.xy % vec2<u32>(16u) == vec2<u32>(0u)) {
        let luminances0 = previous_mip_luminances[local_id.x + 0u][local_id.y + 0u];
        let luminances1 = previous_mip_luminances[local_id.x + 8u][local_id.y + 0u];
        let luminances2 = previous_mip_luminances[local_id.x + 0u][local_id.y + 8u];
        let luminances3 = previous_mip_luminances[local_id.x + 8u][local_id.y + 8u];
        let luminances = (luminances0 + luminances1 + luminances2 + luminances3) / 4.0;

        let weight0 = previous_mip_weights[local_id.x + 0u][local_id.y + 0u];
        let weight1 = previous_mip_weights[local_id.x + 8u][local_id.y + 0u];
        let weight2 = previous_mip_weights[local_id.x + 0u][local_id.y + 8u];
        let weight3 = previous_mip_weights[local_id.x + 8u][local_id.y + 8u];
        let weight = (weight0 + weight1 + weight2 + weight3) / 4.0;

        textureStore(luminances_image_mip5, base_coordinates / 16i, vec4(luminances, 1.0));
        textureStore(weights_image_mip5, base_coordinates / 16i, vec4(weight, 1.0));
    }
}
