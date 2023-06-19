use super::{
    view_resources::SolariBindGroupLayout, world_cache::resources::SolariWorldCacheResources, *,
};
use crate::{scene::bind_group_layout::SolariSceneResources, SolariDebugView, SolariSettings};
use bevy_ecs::{
    prelude::{Component, Entity},
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::render_resource::{
    BindGroupLayout, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache,
    ShaderDefVal, SpecializedComputePipeline, SpecializedComputePipelines,
};

#[derive(Resource)]
pub struct SolariPipelines {
    scene_bind_group_layout: BindGroupLayout,
    bind_group_layout: BindGroupLayout,
    world_cache_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariPipelines {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        let bind_group_layout = world.resource::<SolariBindGroupLayout>();
        let world_cache_resources = world.resource::<SolariWorldCacheResources>();

        Self {
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
            bind_group_layout: bind_group_layout.0.clone(),
            world_cache_bind_group_layout: world_cache_resources.bind_group_layout.clone(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum SolariPipelinesKey {
    GmtBuffer,
    UpdateScreenProbes,
    FilterScreenProbes,
    InterpolateScreenProbes,
    DenoiseIndirectDiffuseTemporal,
    DenoiseIndirectDiffuseSpatial,
    ShadeViewTarget { debug_view: Option<SolariDebugView> },
    Taa,
}

impl SpecializedComputePipeline for SolariPipelines {
    type Key = SolariPipelinesKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        let (entry_point, label, shader) = match key {
            SolariPipelinesKey::GmtBuffer => (
                "gmt_buffer",
                "solari_gmt_buffer_pipeline",
                SOLARI_GMT_BUFFER_SHADER,
            ),
            SolariPipelinesKey::UpdateScreenProbes => (
                "update_screen_probes",
                "solari_update_screen_probes_pipeline",
                SOLARI_UPDATE_SCREEN_PROBES_SHADER,
            ),
            SolariPipelinesKey::FilterScreenProbes => (
                "filter_screen_probes",
                "solari_filter_screen_probes_pipeline",
                SOLARI_FILTER_SCREEN_PROBES_SHADER,
            ),
            SolariPipelinesKey::InterpolateScreenProbes => (
                "interpolate_screen_probes",
                "solari_interpolate_screen_probes_pipeline",
                SOLARI_INTEPOLATE_SCREEN_PROBES_SHADER,
            ),
            SolariPipelinesKey::DenoiseIndirectDiffuseTemporal => (
                "denoise_indirect_diffuse_temporal",
                "solari_denoise_indirect_diffuse_temporal_pipeline",
                SOLARI_DENOISE_INDIRECT_DIFFUSE_SHADER,
            ),
            SolariPipelinesKey::DenoiseIndirectDiffuseSpatial => (
                "denoise_indirect_diffuse_spatial",
                "solari_denoise_indirect_diffuse_spatial_pipeline",
                SOLARI_DENOISE_INDIRECT_DIFFUSE_SHADER,
            ),
            SolariPipelinesKey::ShadeViewTarget { .. } => (
                "shade_view_target",
                "solari_shade_view_target_pipeline",
                SOLARI_SHADE_VIEW_TARGET_SHADER,
            ),
            SolariPipelinesKey::Taa => ("taa", "solari_taa_pipeline", SOLARI_TAA_SHADER),
        };

        let mut shader_defs = vec![ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 2)];
        if let SolariPipelinesKey::ShadeViewTarget {
            debug_view: Some(debug_view),
        } = key
        {
            let shader_def = match debug_view {
                SolariDebugView::Depth => "DEBUG_VIEW_DEPTH",
                SolariDebugView::WorldNormals => "DEBUG_VIEW_WORLD_NORMALS",
                SolariDebugView::MotionVectors => "DEBUG_VIEW_MOTION_VECTORS",
                SolariDebugView::BaseColors => "DEBUG_VIEW_BASE_COLORS",
                SolariDebugView::WorldCacheIrradiance => "DEBUG_VIEW_WORLD_CACHE_IRRADIANCE",
                SolariDebugView::ScreenProbesUnfiltered => "DEBUG_VIEW_SCREEN_PROBES_UNFILTERED",
                SolariDebugView::ScreenProbesFiltered => "DEBUG_VIEW_SCREEN_PROBES_FILTERED",
                SolariDebugView::DirectLight => "DEBUG_VIEW_DIRECT_LIGHT",
                SolariDebugView::IndirectLight => "DEBUG_VIEW_INDIRECT_LIGHT",
            };
            shader_defs.push(shader_def.into());
        }

        ComputePipelineDescriptor {
            label: Some(label.into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.bind_group_layout.clone(),
                self.world_cache_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: shader.typed(),
            shader_defs,
            entry_point: entry_point.into(),
        }
    }
}

#[derive(Component)]
pub struct SolariPipelineIds {
    pub gmt_buffer: CachedComputePipelineId,
    pub update_screen_probes: CachedComputePipelineId,
    pub filter_screen_probes: CachedComputePipelineId,
    pub interpolate_screen_probes: CachedComputePipelineId,
    pub denoise_indirect_diffuse_temporal: CachedComputePipelineId,
    pub denoise_indirect_diffuse_spatial: CachedComputePipelineId,
    pub shade_view_target: CachedComputePipelineId,
    pub taa: CachedComputePipelineId,
}

pub fn prepare_pipelines(
    views: Query<(Entity, &SolariSettings)>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariPipelines>>,
    pipeline: Res<SolariPipelines>,
) {
    let mut create_pipeline = |key| pipelines.specialize(&pipeline_cache, &pipeline, key);

    for (entity, solari_settings) in &views {
        commands.entity(entity).insert(SolariPipelineIds {
            gmt_buffer: create_pipeline(SolariPipelinesKey::GmtBuffer),
            update_screen_probes: create_pipeline(SolariPipelinesKey::UpdateScreenProbes),
            filter_screen_probes: create_pipeline(SolariPipelinesKey::FilterScreenProbes),
            interpolate_screen_probes: create_pipeline(SolariPipelinesKey::InterpolateScreenProbes),
            denoise_indirect_diffuse_temporal: create_pipeline(
                SolariPipelinesKey::DenoiseIndirectDiffuseTemporal,
            ),
            denoise_indirect_diffuse_spatial: create_pipeline(
                SolariPipelinesKey::DenoiseIndirectDiffuseSpatial,
            ),
            shade_view_target: create_pipeline(SolariPipelinesKey::ShadeViewTarget {
                debug_view: solari_settings.debug_view,
            }),
            taa: create_pipeline(SolariPipelinesKey::Taa),
        });
    }
}
