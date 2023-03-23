#define_import_path bevy_sprite::mesh2d_bindings

#import bevy_sprite::mesh2d_types

@group(2) @binding(0)
#if AVAILABLE_STORAGE_BUFFER_BINDINGS >= 3
var<storage> mesh: array<Mesh>;
#else
var<uniform> mesh: array<Mesh2d>;
#endif
