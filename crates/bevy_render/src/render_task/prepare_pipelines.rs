use super::{prepare_bind_groups::create_bind_group_layouts, RenderTask};
use crate::render_resource::{
    BindGroupLayout, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache,
    SpecializedComputePipeline, SpecializedComputePipelines,
};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_utils::HashMap;
use std::marker::PhantomData;

#[derive(Resource)]
pub struct RenderTaskPipelines<R: RenderTask> {
    pub bind_group_layouts: HashMap<&'static str, BindGroupLayout>,
    _marker: PhantomData<R>,
}

impl<R: RenderTask> FromWorld for RenderTaskPipelines<R> {
    fn from_world(world: &mut World) -> Self {
        Self {
            bind_group_layouts: create_bind_group_layouts::<R>(world.resource()),
            _marker: PhantomData,
        }
    }
}

impl<R: RenderTask> RenderTaskPipelines<R> {
    pub fn prepare_pipelines(
        mut commands: Commands,
        pipeline_cache: Res<PipelineCache>,
        mut special_pipeline: ResMut<SpecializedComputePipelines<Self>>,
        pipeline: Res<Self>,
        query: Query<Entity, With<R::RenderTaskSettings>>,
    ) {
        for entity in &query {
            let mut pipeline_ids = HashMap::new();

            for pass_name in R::passes().keys() {
                pipeline_ids.insert(
                    *pass_name,
                    special_pipeline.specialize(&pipeline_cache, &pipeline, pass_name),
                );
            }

            commands.entity(entity).insert(RenderTaskPipelineIds::<R> {
                ids: pipeline_ids,
                _marker: PhantomData,
            });
        }
    }
}

impl<R: RenderTask> SpecializedComputePipeline for RenderTaskPipelines<R> {
    type Key = &'static str;

    fn specialize(&self, pass_name: Self::Key) -> ComputePipelineDescriptor {
        let pass = &R::passes()[pass_name];

        ComputePipelineDescriptor {
            label: Some(format!("{}_{pass_name}", R::name()).into()),
            layout: vec![self.bind_group_layouts[pass_name].clone()],
            push_constant_ranges: vec![],
            shader: pass.shader.clone(),
            shader_defs: pass.shader_defs.to_vec(),
            entry_point: pass.entry_point.unwrap_or(pass_name).into(),
        }
    }
}

#[derive(Component)]
pub struct RenderTaskPipelineIds<R: RenderTask> {
    pub ids: HashMap<&'static str, CachedComputePipelineId>,
    _marker: PhantomData<R>,
}
