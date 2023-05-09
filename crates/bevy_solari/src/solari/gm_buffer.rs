use super::{
    resources::SolariBindGroupLayout, world_cache::resources::SolariWorldCacheResources,
    SOLARI_GM_BUFFER_SHADER,
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
pub struct SolariGmBufferPipeline {
    scene_bind_group_layout: BindGroupLayout,
    bind_group_layout: BindGroupLayout,
    world_cache_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariGmBufferPipeline {
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
pub struct SolariGmBufferPipelineKey {}

impl SpecializedComputePipeline for SolariGmBufferPipeline {
    type Key = SolariGmBufferPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_gm_buffer_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.bind_group_layout.clone(),
                self.world_cache_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_GM_BUFFER_SHADER.typed(),
            shader_defs: vec![ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 2)],
            entry_point: "gm_buffer".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariGmBufferPipelineId(pub CachedComputePipelineId);

pub fn prepare_gm_buffer_pipelines(
    views: Query<Entity, With<SolariSettings>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariGmBufferPipeline>>,
    pipeline: Res<SolariGmBufferPipeline>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariGmBufferPipelineKey {});

        commands
            .entity(entity)
            .insert(SolariGmBufferPipelineId(pipeline_id));
    }
}
