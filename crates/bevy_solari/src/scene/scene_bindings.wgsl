#define_import_path bevy_solari::scene_bindings

#import bevy_solari::scene_types
#import bevy_render::globals

@group(0) @binding(0) var tlas: acceleration_structure;
@group(0) @binding(1) var<storage> index_buffers: binding_array<SolariIndexBuffer>;
@group(0) @binding(2) var<storage> vertex_buffers: binding_array<SolariVertexBuffer>;
@group(0) @binding(3) var<storage> previous_transforms: array<mat4x4<f32>>;
@group(0) @binding(4) var<storage> materials: array<SolariMaterial>;
@group(0) @binding(5) var texture_maps: binding_array<texture_2d<f32>>;
@group(0) @binding(6) var texture_sampler: sampler;
@group(0) @binding(7) var<storage> emissive_object_mesh_material_indices: array<u32>;
@group(0) @binding(8) var<storage> emissive_object_transforms: array<mat4x4<f32>>;
@group(0) @binding(9) var<storage> emissive_object_triangle_counts: array<u32>;
@group(0) @binding(10) var<uniform> globals: Globals;
