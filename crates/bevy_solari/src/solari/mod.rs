pub mod camera;
mod filter_screen_probes;
mod gm_buffer;
pub mod node;
mod resources;
mod update_screen_probes;

use self::{
    camera::SolariSettings,
    filter_screen_probes::{
        prepare_filter_screen_probe_pipelines, SolariFilterScreenProbesPipeline,
    },
    gm_buffer::{prepare_gm_buffer_pipelines, SolariGmBufferPipeline},
    resources::{prepare_resources, queue_bind_groups},
    update_screen_probes::{
        prepare_update_screen_probe_pipelines, SolariUpdateScreenProbesPipeline,
    },
};
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_reflect::TypeUuid;
use bevy_render::{
    extract_component::ExtractComponentPlugin,
    render_resource::{Shader, SpecializedComputePipelines},
    Render, RenderApp, RenderSet,
};

pub struct SolariRealtimePlugin;

const SOLARI_GM_BUFFER_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5717171717171755);
const SOLARI_UPDATE_SCREEN_PROBES_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6717171717171755);
const SOLARI_FILTER_SCREEN_PROBES_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7717171717171755);

impl Plugin for SolariRealtimePlugin {
    fn build(&self, app: &mut App) {
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

        app.add_plugin(ExtractComponentPlugin::<SolariSettings>::default());

        app.sub_app_mut(RenderApp)
            .init_resource::<SolariGmBufferPipeline>()
            .init_resource::<SolariUpdateScreenProbesPipeline>()
            .init_resource::<SolariFilterScreenProbesPipeline>()
            .init_resource::<SpecializedComputePipelines<SolariGmBufferPipeline>>()
            .init_resource::<SpecializedComputePipelines<SolariUpdateScreenProbesPipeline>>()
            .init_resource::<SpecializedComputePipelines<SolariFilterScreenProbesPipeline>>()
            .add_systems(
                Render,
                (
                    prepare_resources,
                    prepare_gm_buffer_pipelines,
                    prepare_update_screen_probe_pipelines,
                    prepare_filter_screen_probe_pipelines,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue_bind_groups.in_set(RenderSet::Queue));
    }
}
