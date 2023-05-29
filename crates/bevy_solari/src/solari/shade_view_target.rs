use super::{
    resources::SolariBindGroupLayout, world_cache::resources::SolariWorldCacheResources,
    SOLARI_SHADE_VIEW_TARGET,
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
pub struct SolariShadeViewTargetPipeline {
    scene_bind_group_layout: BindGroupLayout,
    bind_group_layout: BindGroupLayout,
    world_cache_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariShadeViewTargetPipeline {
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
pub struct SolariShadeViewTargetPipelineKey {
    debug_view: Option<SolariDebugView>,
}

impl SpecializedComputePipeline for SolariShadeViewTargetPipeline {
    type Key = SolariShadeViewTargetPipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        let mut shader_defs = vec![ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 2)];
        match key.debug_view {
            Some(SolariDebugView::Depth) => shader_defs.push("DEBUG_VIEW_DEPTH".into()),
            Some(SolariDebugView::WorldNormals) => {
                shader_defs.push("DEBUG_VIEW_WORLD_NORMALS".into())
            }
            Some(SolariDebugView::MotionVectors) => {
                shader_defs.push("DEBUG_VIEW_MOTION_VECTORS".into())
            }
            Some(SolariDebugView::BaseColors) => shader_defs.push("DEBUG_VIEW_BASE_COLORS".into()),
            Some(SolariDebugView::Irradiance) => shader_defs.push("DEBUG_VIEW_IRRADIANCE".into()),
            Some(SolariDebugView::ScreenProbesUnfiltered) => {
                shader_defs.push("DEBUG_VIEW_SCREEN_PROBES_UNFILTERED".into())
            }
            Some(SolariDebugView::ScreenProbesFiltered) => {
                shader_defs.push("DEBUG_VIEW_SCREEN_PROBES_FILTERED".into())
            }
            Some(SolariDebugView::WorldCacheIrradiance) => {
                shader_defs.push("DEBUG_VIEW_WORLD_CACHE_IRRADIANCE".into())
            }
            None => {}
        }

        ComputePipelineDescriptor {
            label: Some("solari_shade_view_target_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.bind_group_layout.clone(),
                self.world_cache_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SHADE_VIEW_TARGET.typed(),
            shader_defs,
            entry_point: "shade_view_target".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariShadeViewTargetPipelineId(pub CachedComputePipelineId);

pub fn prepare_shade_view_target_pipelines(
    views: Query<(Entity, &SolariSettings)>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariShadeViewTargetPipeline>>,
    pipeline: Res<SolariShadeViewTargetPipeline>,
) {
    for (entity, solari_settings) in &views {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            SolariShadeViewTargetPipelineKey {
                debug_view: solari_settings.debug_view,
            },
        );

        commands
            .entity(entity)
            .insert(SolariShadeViewTargetPipelineId(pipeline_id));
    }
}
