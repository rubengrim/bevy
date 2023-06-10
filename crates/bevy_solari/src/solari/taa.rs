use super::{
    resources::SolariBindGroupLayout, world_cache::resources::SolariWorldCacheResources,
    SOLARI_TAA_SHADER,
};
use crate::{scene::bind_group_layout::SolariSceneResources, SolariSettings};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::render_resource::{
    BindGroupLayout, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache,
    ShaderDefVal, SpecializedComputePipeline, SpecializedComputePipelines,
};

#[derive(Resource)]
pub struct SolariTaaPipeline {
    scene_bind_group_layout: BindGroupLayout,
    bind_group_layout: BindGroupLayout,
    world_cache_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariTaaPipeline {
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
pub struct SolariTaaPipelineKey {}

impl SpecializedComputePipeline for SolariTaaPipeline {
    type Key = SolariTaaPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_taa_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.bind_group_layout.clone(),
                self.world_cache_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_TAA_SHADER.typed(),
            shader_defs: vec![ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 2)],
            entry_point: "taa".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariTaaPipelineId(pub CachedComputePipelineId);

pub fn prepare_taa_pipelines(
    views: Query<Entity, With<SolariSettings>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariTaaPipeline>>,
    pipeline: Res<SolariTaaPipeline>,
) {
    for entity in &views {
        let pipeline_id = pipelines.specialize(&pipeline_cache, &pipeline, SolariTaaPipelineKey {});

        commands
            .entity(entity)
            .insert(SolariTaaPipelineId(pipeline_id));
    }
}
