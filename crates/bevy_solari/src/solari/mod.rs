pub mod camera;
mod filter_screen_probes;
mod gm_buffer;
pub mod node;
mod resources;
mod shade_view_target;
mod update_screen_probes;
pub mod world_cache;

use self::{
    camera::{
        prepare_previous_view_projection_uniforms, update_previous_view_projections,
        PreviousViewProjectionUniforms, SolariSettings,
    },
    filter_screen_probes::{
        prepare_filter_screen_probe_pipelines, SolariFilterScreenProbesPipeline,
    },
    gm_buffer::{prepare_gm_buffer_pipelines, SolariGmBufferPipeline},
    resources::{prepare_resources, queue_bind_groups, SolariBindGroupLayout},
    shade_view_target::{prepare_shade_view_target_pipelines, SolariShadeViewTargetPipeline},
    update_screen_probes::{
        prepare_update_screen_probe_pipelines, SolariUpdateScreenProbesPipeline,
    },
    world_cache::SolariWorldCachePlugin,
};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_reflect::TypeUuid;
use bevy_render::{
    extract_component::ExtractComponentPlugin,
    render_resource::{Shader, SpecializedComputePipelines},
    Render, RenderApp, RenderSet,
};

pub struct SolariRealtimePlugin;

const SOLARI_VIEW_BINDINGS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5717171717171755);
const SOLARI_GM_BUFFER_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6717171717171755);
const SOLARI_UPDATE_SCREEN_PROBES_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7717171717171755);
const SOLARI_FILTER_SCREEN_PROBES_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 8717171717171755);
const SOLARI_SHADE_VIEW_TARGET: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9717171717171755);

impl Plugin for SolariRealtimePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SOLARI_VIEW_BINDINGS_SHADER,
            "view_bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_GM_BUFFER_SHADER,
            "gm_buffer.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_UPDATE_SCREEN_PROBES_SHADER,
            "update_screen_probes.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_FILTER_SCREEN_PROBES_SHADER,
            "filter_screen_probes.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_SHADE_VIEW_TARGET,
            "shade_view_target.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(SolariWorldCachePlugin)
            .add_plugin(ExtractComponentPlugin::<SolariSettings>::default())
            .add_systems(PreUpdate, update_previous_view_projections);

        app.sub_app_mut(RenderApp)
            .init_resource::<PreviousViewProjectionUniforms>()
            .init_resource::<SolariBindGroupLayout>()
            .init_resource::<SolariGmBufferPipeline>()
            .init_resource::<SolariUpdateScreenProbesPipeline>()
            .init_resource::<SolariFilterScreenProbesPipeline>()
            .init_resource::<SolariShadeViewTargetPipeline>()
            .init_resource::<SpecializedComputePipelines<SolariGmBufferPipeline>>()
            .init_resource::<SpecializedComputePipelines<SolariUpdateScreenProbesPipeline>>()
            .init_resource::<SpecializedComputePipelines<SolariFilterScreenProbesPipeline>>()
            .init_resource::<SpecializedComputePipelines<SolariShadeViewTargetPipeline>>()
            .add_systems(
                Render,
                (
                    prepare_previous_view_projection_uniforms,
                    prepare_resources,
                    prepare_gm_buffer_pipelines,
                    prepare_update_screen_probe_pipelines,
                    prepare_filter_screen_probe_pipelines,
                    prepare_shade_view_target_pipelines,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue_bind_groups.in_set(RenderSet::Queue));
    }
}
