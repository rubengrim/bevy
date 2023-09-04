pub mod camera;
pub mod node;
mod pipelines;
mod view_resources;
mod world_cache;

use self::{
    camera::{
        prepare_previous_view_projection_uniforms, prepare_taa_jitter,
        update_previous_view_projections, PreviousViewProjection, PreviousViewProjectionUniforms,
        SolariSettings,
    },
    pipelines::{prepare_pipelines, SolariPipelines},
    view_resources::{prepare_view_resources, queue_view_bind_groups, SolariViewBindGroupLayout},
    world_cache::{pipelines::SolariWorldCachePipelineIds, resources::SolariWorldCacheResources},
};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_reflect::TypeUuid;
use bevy_render::{
    extract_component::ExtractComponentPlugin,
    render_resource::{Shader, SpecializedComputePipelines},
    view::prepare_view_uniforms,
    Render, RenderApp, RenderSet,
};

pub struct SolariRealtimePlugin;

const SOLARI_VIEW_BINDINGS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5717171717171755);
const SOLARI_GMT_BUFFER_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6717171717171755);
const SOLARI_UPDATE_SCREEN_PROBES_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7717171717171755);
const SOLARI_FILTER_SCREEN_PROBES_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 8717171717171755);
const SOLARI_INTEPOLATE_SCREEN_PROBES_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9717171717171755);
const SOLARI_DENOISE_INDIRECT_DIFFUSE_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1617171717171755);
const SOLARI_SAMPLE_DIRECT_DIFFUSE_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1617171717171756);
const SOLARI_DENOISE_DIRECT_DIFFUSE_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1617171717171757);
const SOLARI_SHADE_VIEW_TARGET_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1617171717171758);
const SOLARI_TAA_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1617171717171759);
const SOLARI_WORLD_CACHE_BINDINGS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1717171717171756);
const SOLARI_WORLD_CACHE_QUERY_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2717171717171756);
const SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3717171717171756);
const SOLARI_WORLD_CACHE_UPDATE_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4717171717171756);

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
            SOLARI_GMT_BUFFER_SHADER,
            "gmt_buffer.wgsl",
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
            SOLARI_INTEPOLATE_SCREEN_PROBES_SHADER,
            "interpolate_screen_probes.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_DENOISE_INDIRECT_DIFFUSE_SHADER,
            "denoise_indirect_diffuse.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SOLARI_SAMPLE_DIRECT_DIFFUSE_SHADER,
            "sample_direct_diffuse.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_DENOISE_DIRECT_DIFFUSE_SHADER,
            "denoise_direct_diffuse.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_SHADE_VIEW_TARGET_SHADER,
            "shade_view_target.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(app, SOLARI_TAA_SHADER, "taa.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            SOLARI_WORLD_CACHE_BINDINGS_SHADER,
            "world_cache/world_cache_bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_WORLD_CACHE_QUERY_SHADER,
            "world_cache/world_cache_query.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER,
            "world_cache/compact_active_cells.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_WORLD_CACHE_UPDATE_SHADER,
            "world_cache/update_world_cache.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(ExtractComponentPlugin::<SolariSettings>::default())
            .add_plugin(ExtractComponentPlugin::<PreviousViewProjection>::default())
            .add_systems(PreUpdate, update_previous_view_projections);

        app.sub_app_mut(RenderApp)
            .init_resource::<PreviousViewProjectionUniforms>()
            .init_resource::<SolariViewBindGroupLayout>()
            .init_resource::<SolariWorldCacheResources>()
            .init_resource::<SolariWorldCachePipelineIds>()
            .init_resource::<SolariPipelines>()
            .init_resource::<SpecializedComputePipelines<SolariPipelines>>()
            .add_systems(
                Render,
                (
                    prepare_taa_jitter.before(prepare_view_uniforms),
                    prepare_previous_view_projection_uniforms,
                    prepare_view_resources,
                    prepare_pipelines,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue_view_bind_groups.in_set(RenderSet::Queue));
    }
}
