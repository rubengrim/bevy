#import bevy_solari::global_illumination::view_bindings view, depth_buffer, normals_buffer, screen_probes_spherical_harmonics, screen_probes_a, diffuse_raw
#import bevy_solari::utils depth_to_world_position

// TODO: Plane distance / tile size weights, relaxed interpolation?
// TODO: Jitter interpolation?

@compute @workgroup_size(8, 8, 1)
fn interpolate_screen_probes(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(num_workgroups) workgroup_count: vec3<u32>,
) {
    let screen_size = vec2<u32>(view.viewport.zw);
    if any(global_id.xy >= screen_size) { return; }

    let pixel_depth = view.projection[3][2] / textureLoad(depth_buffer, global_id.xy, 0i);
    if pixel_depth == 0.0 {
        textureStore(diffuse_raw, global_id.xy, vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }
    let pixel_uv = (vec2<f32>(global_id.xy) + 0.5) / view.viewport.zw;
    let pixel_world_position = depth_to_world_position(pixel_depth, pixel_uv);
    let pixel_world_normal = normalize(textureLoad(normals_buffer, global_id.xy, 0i).xyz * 2.0 - vec3(1.0));

    let probe_count = textureDimensions(screen_probes_a) / 8u;
    let probe_id_f = pixel_uv * vec2<f32>(probe_count) - 0.5;

    let tl_probe_id = max(vec2<u32>(probe_id_f), vec2(0u));
    let tr_probe_id = min(tl_probe_id + vec2(1u, 0u), probe_count);
    let bl_probe_id = min(tl_probe_id + vec2(0u, 1u), probe_count);
    let br_probe_id = min(tl_probe_id + vec2(1u, 1u), probe_count);

    let tl_probe_sample = sample_probe(tl_probe_id, pixel_world_normal, probe_count);
    let tr_probe_sample = sample_probe(tr_probe_id, pixel_world_normal, probe_count);
    let bl_probe_sample = sample_probe(bl_probe_id, pixel_world_normal, probe_count);
    let br_probe_sample = sample_probe(tr_probe_id, pixel_world_normal, probe_count);

    let r = fract(probe_id_f);
    let tl_probe_weight = (1.0 - r.x) * (1.0 - r.y);
    let tr_probe_weight = r.x * (1.0 - r.y);
    let bl_probe_weight = (1.0 - r.x) * r.y;
    let br_probe_weight = r.x * r.y;

    let irradiance = (tl_probe_sample * tl_probe_weight) + (tr_probe_sample * tr_probe_weight) + (bl_probe_sample * bl_probe_weight) + (br_probe_sample * br_probe_weight);

    textureStore(diffuse_raw, global_id.xy, vec4(irradiance, 1.0));
}

fn sample_probe(probe_id: vec2<u32>, pixel_world_normal: vec3<f32>, probe_count: vec2<u32>) -> vec3<f32> {
    let c1 = 0.429043;
    let c2 = 0.511664;
    let c3 = 0.743125;
    let c4 = 0.886227;
    let c5 = 0.247708;
    let x = pixel_world_normal.x;
    let y = pixel_world_normal.y;
    let z = pixel_world_normal.z;
    let xz = x * z;
    let yz = y * z;
    let xy = x * y;
    let zz = z * z;
    let xx_yy = x * x - y * y;

    let sh_index = probe_id.x + probe_id.y * probe_count.x;
    let sh = screen_probes_spherical_harmonics[sh_index];

    let L00 = sh.a.xyz;
    let L11 = vec3(sh.a.w, sh.b.xy);
    let L10 = vec3(sh.b.zw, sh.c.x);
    let L1_1 = sh.c.yzw;
    let L21 = sh.d.xyz;
    let L2_1 = vec3(sh.d.w, sh.e.xy);
    let L2_2 = vec3(sh.e.zw, sh.f.x);
    let L20 = sh.f.yzw;
    let L22 = sh.g;
    return (c1 * L22 * xx_yy) + (c3 * L20 * zz) + (c4 * L00) - (c5 * L20) + (2.0 * c1 * ((L2_2 * xy) + (L21 * xz) + (L2_1 * yz))) + (2.0 * c2 * ((L11 * x) + (L1_1 * y) + (L10 * z)));
}
