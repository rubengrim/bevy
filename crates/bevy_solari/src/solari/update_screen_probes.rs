use super::{resources::SolariBindGroupLayout, SOLARI_UPDATE_SCREEN_PROBES_SHADER};
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
pub struct SolariUpdateScreenProbesPipeline {
    scene_bind_group_layout: BindGroupLayout,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariUpdateScreenProbesPipeline {
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
pub struct SolariUpdateScreenProbesPipelineKey {}

impl SpecializedComputePipeline for SolariUpdateScreenProbesPipeline {
    type Key = SolariUpdateScreenProbesPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_update_screen_probes_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_UPDATE_SCREEN_PROBES_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "update_screen_probes".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariUpdateScreenProbesPipelineId(pub CachedComputePipelineId);

pub fn prepare_update_screen_probe_pipelines(
    views: Query<Entity, With<SolariSettings>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariUpdateScreenProbesPipeline>>,
    pipeline: Res<SolariUpdateScreenProbesPipeline>,
) {
    for entity in &views {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            SolariUpdateScreenProbesPipelineKey {},
        );

        commands
            .entity(entity)
            .insert(SolariUpdateScreenProbesPipelineId(pipeline_id));
    }
}
