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
    // TODO: Remove unnecessary texture write + texture allocation #ifndef DEBUG_VIEW_SCREEN_PROBES_FILTERED
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
    var L00 = (0.282095) * filtered;
    var L11 = (0.488603 * x) * filtered;
    var L10 = (0.488603 * z) * filtered;
    var L1_1 = (0.488603 * y) * filtered;
    var L21 = (1.092548 * xz) * filtered;
    var L2_1 = (1.092548 * yz) * filtered;
    var L2_2 = (1.092548 * xy) * filtered;
    var L20 = (0.946176 * zz - 0.315392) * filtered;
    var L22 = (0.546274 * xx_yy) * filtered;

    // TODO: Replace with subgroup/wave ops when supported
    spherical_harmonics_coefficents[local_index][0] = L00;
    spherical_harmonics_coefficents[local_index][1] = L11;
    spherical_harmonics_coefficents[local_index][2] = L10;
    spherical_harmonics_coefficents[local_index][3] = L1_1;
    spherical_harmonics_coefficents[local_index][4] = L21;
    spherical_harmonics_coefficents[local_index][5] = L2_1;
    spherical_harmonics_coefficents[local_index][6] = L2_2;
    spherical_harmonics_coefficents[local_index][7] = L20;
    spherical_harmonics_coefficents[local_index][8] = L22;
    workgroupBarrier();
    for (var t = 32u; t > 0u; t >>= 1u) {
        if local_index < t {
            spherical_harmonics_coefficents[local_index][0] += spherical_harmonics_coefficents[local_index + t][0];
            spherical_harmonics_coefficents[local_index][1] += spherical_harmonics_coefficents[local_index + t][1];
            spherical_harmonics_coefficents[local_index][2] += spherical_harmonics_coefficents[local_index + t][2];
            spherical_harmonics_coefficents[local_index][3] += spherical_harmonics_coefficents[local_index + t][3];
            spherical_harmonics_coefficents[local_index][4] += spherical_harmonics_coefficents[local_index + t][4];
            spherical_harmonics_coefficents[local_index][5] += spherical_harmonics_coefficents[local_index + t][5];
            spherical_harmonics_coefficents[local_index][6] += spherical_harmonics_coefficents[local_index + t][6];
            spherical_harmonics_coefficents[local_index][7] += spherical_harmonics_coefficents[local_index + t][7];
            spherical_harmonics_coefficents[local_index][8] += spherical_harmonics_coefficents[local_index + t][8];
        }
        workgroupBarrier();
    }
    if local_index == 0u {
        L00 = spherical_harmonics_coefficents[0][0] / 64.0;
        L11 = spherical_harmonics_coefficents[0][1] / 64.0;
        L10 = spherical_harmonics_coefficents[0][2] / 64.0;
        L1_1 = spherical_harmonics_coefficents[0][3] / 64.0;
        L21 = spherical_harmonics_coefficents[0][4] / 64.0;
        L2_1 = spherical_harmonics_coefficents[0][5] / 64.0;
        L2_2 = spherical_harmonics_coefficents[0][6] / 64.0;
        L20 = spherical_harmonics_coefficents[0][7] / 64.0;
        L22 = spherical_harmonics_coefficents[0][8] / 64.0;
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
