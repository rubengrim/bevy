use super::{
    resources::SolariBindGroupLayout, world_cache::resources::SolariWorldCacheResources,
    SOLARI_GMT_BUFFER_SHADER,
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
pub struct SolariGmtBufferPipeline {
    scene_bind_group_layout: BindGroupLayout,
    bind_group_layout: BindGroupLayout,
    world_cache_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariGmtBufferPipeline {
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
pub struct SolariGmtBufferPipelineKey {}

impl SpecializedComputePipeline for SolariGmtBufferPipeline {
    type Key = SolariGmtBufferPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_gmt_buffer_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.bind_group_layout.clone(),
                self.world_cache_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_GMT_BUFFER_SHADER.typed(),
            shader_defs: vec![ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 2)],
            entry_point: "gmt_buffer".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariGmtBufferPipelineId(pub CachedComputePipelineId);

pub fn prepare_gmt_buffer_pipelines(
    views: Query<Entity, With<SolariSettings>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariGmtBufferPipeline>>,
    pipeline: Res<SolariGmtBufferPipeline>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariGmtBufferPipelineKey {});

        commands
            .entity(entity)
            .insert(SolariGmtBufferPipelineId(pipeline_id));
    }
}
