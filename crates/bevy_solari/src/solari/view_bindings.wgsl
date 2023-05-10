#define_import_path bevy_solari::view_bindings

#import bevy_render::view
#import bevy_render::globals

struct SphericalHarmonicsPacked {
    b0: vec4<f32>,
    b1: vec4<f32>,
    b2: vec4<f32>,
    b3: vec4<f32>,
    b4: vec4<f32>,
    b5: vec4<f32>,
    b6: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var<uniform> globals: Globals;
@group(1) @binding(2)
var g_buffer: texture_storage_2d<rgba16uint, read_write>;
@group(1) @binding(3)
var m_buffer: texture_storage_2d<rgba16uint, read_write>;
@group(1) @binding(4)
var screen_probes_unfiltered: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(5)
var screen_probes_filtered: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(6)
var<storage, read_write> screen_probe_spherical_harmonics: array<SphericalHarmonicsPacked>;
@group(1) @binding(7)
var view_target: texture_storage_2d<rgba16float, write>;