#define_import_path bevy_pbr::prepass_bindings

#import bevy_pbr::mesh_view_types
#import bevy_pbr::mesh_types

@group(0) @binding(0)
var<uniform> view: View;

@group(0) @binding(1)
var<uniform> globals: Globals;

// Material bindings will be in @group(1)

@group(2) @binding(0)
#if AVAILABLE_STORAGE_BUFFER_BINDINGS >= 3
var<storage> mesh: array<Mesh>;
#else
var<uniform> mesh: array<Mesh>;
#endif

#ifdef SKINNED
@group(2) @binding(1)
var<uniform> joint_matrices: SkinnedMesh;
#import bevy_pbr::skinning
#endif
