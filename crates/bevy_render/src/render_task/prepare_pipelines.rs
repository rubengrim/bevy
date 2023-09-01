use super::RenderTask;
use crate::render_resource::{
    BindGroupLayout, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache,
    SpecializedComputePipeline, SpecializedComputePipelines,
};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_utils::HashMap;
use std::marker::PhantomData;

#[derive(Resource)]
pub struct RenderTaskPipelinesResource<R: RenderTask> {
    bind_group_layouts: HashMap<&'static str, BindGroupLayout>,
    _marker: PhantomData<R>,
}

impl<R: RenderTask> RenderTaskPipelinesResource<R> {
    pub fn new() -> Self {
        Self {
            bind_group_layouts: todo!(),
            _marker: PhantomData,
        }
    }

    pub fn prepare_pipelines(
        mut commands: Commands,
        pipeline_cache: Res<PipelineCache>,
        mut special_pipeline: ResMut<SpecializedComputePipelines<Self>>,
        pipeline: Res<Self>,
        query: Query<Entity, With<R::RenderTaskSettings>>,
    ) {
        for entity in &query {
            let mut pipeline_ids = HashMap::new();

            for key in R::pipelines().keys() {
                let pipeline_id = special_pipeline.specialize(&pipeline_cache, &pipeline, key);
                pipeline_ids.insert(*key, pipeline_id);
            }

            commands.entity(entity).insert(RenderTaskPipelineIds::<R> {
                ids: pipeline_ids,
                _marker: PhantomData,
            });
        }
    }
}

impl<R: RenderTask> SpecializedComputePipeline for RenderTaskPipelinesResource<R> {
    type Key = &'static str;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        let render_task_pipeline = &R::pipelines()[key];

        ComputePipelineDescriptor {
            label: Some(key.into()),
            layout: vec![self.bind_group_layouts[key].clone()],
            push_constant_ranges: vec![],
            shader: render_task_pipeline.shader.clone(),
            shader_defs: vec![], // TODO: Allow the user to specialize their shaders
            entry_point: render_task_pipeline.entry_point.unwrap_or(key).into(),
        }
    }
}

#[derive(Component)]
pub struct RenderTaskPipelineIds<R: RenderTask> {
    pub ids: HashMap<&'static str, CachedComputePipelineId>,
    _marker: PhantomData<R>,
}
