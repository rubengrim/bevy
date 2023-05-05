use super::{resources::SolariBindGroupLayout, SOLARI_SHADE_VIEW_TARGET};
use crate::{scene::bind_group_layout::SolariSceneResources, SolariSettings};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::render_resource::{
    BindGroupLayout, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache,
    SpecializedComputePipeline, SpecializedComputePipelines,
};

#[derive(Resource)]
pub struct SolariShadeViewTargetPipeline {
    scene_bind_group_layout: BindGroupLayout,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariShadeViewTargetPipeline {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        let bind_group_layout = world.resource::<SolariBindGroupLayout>();

        Self {
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
            bind_group_layout: bind_group_layout.0.clone(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct SolariShadeViewTargetPipelineKey {}

impl SpecializedComputePipeline for SolariShadeViewTargetPipeline {
    type Key = SolariShadeViewTargetPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_shade_view_target_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SHADE_VIEW_TARGET.typed(),
            shader_defs: vec![],
            entry_point: "shade_view_target".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariShadeViewTargetPipelineId(pub CachedComputePipelineId);

pub fn prepare_shade_view_target_pipelines(
    views: Query<Entity, With<SolariSettings>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariShadeViewTargetPipeline>>,
    pipeline: Res<SolariShadeViewTargetPipeline>,
) {
    for entity in &views {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            SolariShadeViewTargetPipelineKey {},
        );

        commands
            .entity(entity)
            .insert(SolariShadeViewTargetPipelineId(pipeline_id));
    }
}
