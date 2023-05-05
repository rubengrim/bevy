#import bevy_solari::scene_bindings
#import bevy_solari::view_bindings
#import bevy_solari::utils

var<workgroup> spherical_harmonics_coefficents: array<array<vec3<f32>, 9>, 64>;

@compute @workgroup_size(8, 8, 1)
fn filter_screen_probes(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(num_workgroups) workgroup_count: vec3<u32>,
) {
    let probe_index = workgroup_id.x + workgroup_id.y * workgroup_count.x;
    let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);
    let frame_index = globals.frame_count * 5782582u;
    var rng = pixel_index + frame_index;

    // TODO: Validate neighbor screen_probes_unfiltered indices exist
    // TODO: Depth + angle weighted average
    let tl = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(-8i, 8i)).rgb;
    let tm = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(0i, 8i)).rgb * 2.0;
    let tr = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(8i, 8i)).rgb;
    let ml = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(-8i, 0i)).rgb * 2.0;
    let mm = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(0i, 0i)).rgb * 4.0;
    let mr = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(8i, 0i)).rgb * 2.0;
    let bl = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(-8i, -8i)).rgb;
    let bm = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(0i, -8i)).rgb * 2.0;
    let br = textureLoad(screen_probes_unfiltered, vec2<i32>(global_id.xy) + vec2(8i, -8i)).rgb;
    let filtered = (tl + tm + tr + ml + mm + mr + bl + bm + br) / 16.0;
    textureStore(screen_probes_filtered, global_id.xy, vec4(filtered, 1.0));

    let octahedral_pixel_center = vec2<f32>(local_id.xy) + rand_vec2(&rng);
    let octahedral_normal = octahedral_decode(octahedral_pixel_center / 8.0);
    let x = octahedral_normal.x;
    let y = octahedral_normal.y;
    let z = octahedral_normal.z;
    let xz = x * z;
    let yz = y * z;
    let xy = x * y;
    let zz = z * z;
    let xx_yy = x * x - y * y;

    let Y00 = 0.282095;
    let Y11 = 0.488603 * x;
    let Y10 = 0.488603 * z;
    let Y1_1 = 0.488603 * y;
    let Y21 = 1.092548 * xz;
    let Y2_1 = 1.092548 * yz;
    let Y2_2 = 1.092548 * xy;
    let Y20 = 0.946176 * zz - 0.315392;
    let Y22 = 0.546274 * xx_yy;
    spherical_harmonics_coefficents[local_index][0] = filtered * Y00;
    spherical_harmonics_coefficents[local_index][1] = filtered * Y11;
    spherical_harmonics_coefficents[local_index][2] = filtered * Y10;
    spherical_harmonics_coefficents[local_index][3] = filtered * Y1_1;
    spherical_harmonics_coefficents[local_index][4] = filtered * Y21;
    spherical_harmonics_coefficents[local_index][5] = filtered * Y2_1;
    spherical_harmonics_coefficents[local_index][6] = filtered * Y2_2;
    spherical_harmonics_coefficents[local_index][7] = filtered * Y20;
    spherical_harmonics_coefficents[local_index][8] = filtered * Y22;

    workgroupBarrier();

    if local_index == 0u {
        var L00 = vec3(0.0);
        var L11 = vec3(0.0);
        var L10 = vec3(0.0);
        var L1_1 = vec3(0.0);
        var L21 = vec3(0.0);
        var L2_1 = vec3(0.0);
        var L2_2 = vec3(0.0);
        var L20 = vec3(0.0);
        var L22 = vec3(0.0);
        for (var t = 0u; t < 64u; t++) {
            L00 += spherical_harmonics_coefficents[t][0];
            L11 += spherical_harmonics_coefficents[t][1];
            L10 += spherical_harmonics_coefficents[t][2];
            L1_1 += spherical_harmonics_coefficents[t][3];
            L21 += spherical_harmonics_coefficents[t][4];
            L2_1 += spherical_harmonics_coefficents[t][5];
            L2_2 += spherical_harmonics_coefficents[t][6];
            L20 += spherical_harmonics_coefficents[t][7];
            L22 += spherical_harmonics_coefficents[t][8];
        }
        L00 /= 64.0;
        L11 /= 64.0;
        L10 /= 64.0;
        L1_1 /= 64.0;
        L21 /= 64.0;
        L2_1 /= 64.0;
        L2_2 /= 64.0;
        L20 /= 64.0;
        L22 /= 64.0;
        screen_probe_spherical_harmonics[probe_index] = SphericalHarmonicsPacked(
            vec4(L00, L11.x),
            vec4(L11.yz, L10.xy),
            vec4(L10.z, L1_1),
            vec4(L21, L2_1.x),
            vec4(L2_1.yz, L2_2.xy),
            vec4(L2_2.z, L20),
            L22,
        );
    }
}
