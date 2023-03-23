#define_import_path bevy_pbr::mesh_bindings

#import bevy_pbr::mesh_types

@group(2) @binding(0)
#if AVAILABLE_STORAGE_BUFFER_BINDINGS >= 3
var<storage> mesh: Mesh;
#else
var<uniform> mesh: array<Mesh>;
#endif

#ifdef SKINNED
@group(2) @binding(1)
var<uniform> joint_matrices: SkinnedMesh;
#import bevy_pbr::skinning
#endif
