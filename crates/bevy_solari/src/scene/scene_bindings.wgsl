#define_import_path bevy_solari::scene_bindings

#import bevy_solari::scene_types

@group(0) @binding(0)
var tlas: acceleration_structure;
@group(0) @binding(1)
var<storage> mesh_materials: array<SolariMeshMaterial>;
@group(0) @binding(2)
var<storage> index_buffers: binding_array<SolariIndexBuffer>;
@group(0) @binding(3)
var<storage> vertex_buffers: binding_array<SolariVertexBuffer>;
@group(0) @binding(4)
var<storage> materials: array<SolariMaterial>;
@group(0) @binding(5)
var texture_maps: binding_array<texture_2d<f32>>;
@group(0) @binding(6)
var texture_sampler: sampler;
