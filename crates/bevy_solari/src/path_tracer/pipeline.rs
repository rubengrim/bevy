use super::{
    bind_group::create_view_bind_group_layout, camera::SolariPathTracer, SOLARI_PATH_TRACER_SHADER,
};
use crate::scene::bind_group_layout::SolariSceneResources;
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::{
    render_resource::{
        BindGroupLayout, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache,
        PushConstantRange, ShaderStages, SpecializedComputePipeline, SpecializedComputePipelines,
    },
    renderer::RenderDevice,
};

#[derive(Resource)]
pub struct SolariPathtracerPipeline {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariPathtracerPipeline {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct SolariPathTracerPipelineKey {}

impl SpecializedComputePipeline for SolariPathtracerPipeline {
    type Key = SolariPathTracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_path_tracer_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..4,
            }],
            shader: SOLARI_PATH_TRACER_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "path_trace".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariPathTracerPipelineId(pub CachedComputePipelineId);

pub fn prepare_pipelines(
    views: Query<Entity, With<SolariPathTracer>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariPathtracerPipeline>>,
    pipeline: Res<SolariPathtracerPipeline>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariPathTracerPipelineKey {});

        commands
            .entity(entity)
            .insert(SolariPathTracerPipelineId(pipeline_id));
    }
}
