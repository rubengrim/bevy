#define_import_path bevy_solari::view_bindings

#import bevy_render::view

struct SphericalHarmonicsPacked {
    b0: vec4<f32>,
    b1: vec4<f32>,
    b2: vec4<f32>,
    b3: vec4<f32>,
    b4: vec4<f32>,
    b5: vec4<f32>,
    b6: vec3<f32>,
}

@group(1) @binding(0) var<uniform> view: View;
@group(1) @binding(1) var<uniform> previous_view_proj: mat4x4<f32>;
@group(1) @binding(2) var g_buffer_previous: texture_2d<u32>;
@group(1) @binding(3) var g_buffer: texture_storage_2d<rgba16uint, read_write>;
@group(1) @binding(4) var m_buffer: texture_storage_2d<rgba16uint, read_write>;
@group(1) @binding(5) var t_buffer: texture_storage_2d<rg16float, read_write>;
@group(1) @binding(6) var screen_probes_unfiltered: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(7) var screen_probes_filtered: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(8) var<storage, read_write> screen_probes_spherical_harmonics: array<SphericalHarmonicsPacked>;
@group(1) @binding(9) var indirect_diffuse: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(10) var indirect_diffuse_denoiser_temporal_history: texture_2d<f32>;
@group(1) @binding(11) var indirect_diffuse_denoised_temporal: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(12) var indirect_diffuse_denoised_spatiotemporal: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(13) var taa_history: texture_2d<f32>;
@group(1) @binding(14) var taa_history_output: texture_storage_2d<rgba16float, write>;
@group(1) @binding(15) var view_target_other: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(16) var view_target: texture_storage_2d<rgba16float, write>;
