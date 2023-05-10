#import bevy_solari::scene_bindings
#import bevy_render::view
#import bevy_solari::utils

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var accumulation_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(2)
var output_texture: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(3)
var<storage, read_write> rays: array<RayDesc2>;

@compute @workgroup_size(8, 8, 1)
fn trace_rays(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);

    let ray_origin = rays[pixel_index].origin.xyz;
    let ray_direction = rays[pixel_index].direction.xyz;

    let ray_hit = trace_ray(ray_origin, ray_direction, 0.001);
}
