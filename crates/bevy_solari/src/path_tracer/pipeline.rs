use super::{
    camera::SolariPathTracer, resources::create_view_bind_group_layout, SOLARI_PATH_TRACER_SHADER,
    SOLARI_SORT_RAYS_SHADER, SOLARI_TRACE_RAYS_SHADER,
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

#[derive(Resource)]
pub struct TraceRaysFromBuffer {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for TraceRaysFromBuffer {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
        }
    }
}

impl SpecializedComputePipeline for TraceRaysFromBuffer {
    type Key = SolariPathTracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_path_tracer_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_TRACE_RAYS_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "trace_rays".into(),
        }
    }
}

#[derive(Component)]
pub struct TraceRaysFromBufferId(pub CachedComputePipelineId);

pub fn prepare_pipelines2(
    views: Query<Entity, With<SolariPathTracer>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<TraceRaysFromBuffer>>,
    pipeline: Res<TraceRaysFromBuffer>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariPathTracerPipelineKey {});

        commands
            .entity(entity)
            .insert(TraceRaysFromBufferId(pipeline_id));
    }
}

#[derive(Resource)]
pub struct GenerateKey32 {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for GenerateKey32 {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
        }
    }
}

impl SpecializedComputePipeline for GenerateKey32 {
    type Key = SolariPathTracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("generate_key32".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SORT_RAYS_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "generate_key32".into(),
        }
    }
}

#[derive(Component)]
pub struct GenerateKey32Id(pub CachedComputePipelineId);

pub fn prepare_pipelines3(
    views: Query<Entity, With<SolariPathTracer>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<GenerateKey32>>,
    pipeline: Res<GenerateKey32>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariPathTracerPipelineKey {});

        commands.entity(entity).insert(GenerateKey32Id(pipeline_id));
    }
}

#[derive(Resource)]
pub struct CheckOrderKey32 {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for CheckOrderKey32 {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
        }
    }
}

impl SpecializedComputePipeline for CheckOrderKey32 {
    type Key = SolariPathTracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("check_order_key32".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SORT_RAYS_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "check_order_key32".into(),
        }
    }
}

#[derive(Component)]
pub struct CheckOrderKey32Id(pub CachedComputePipelineId);

pub fn prepare_pipelines4(
    views: Query<Entity, With<SolariPathTracer>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<CheckOrderKey32>>,
    pipeline: Res<CheckOrderKey32>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariPathTracerPipelineKey {});

        commands
            .entity(entity)
            .insert(CheckOrderKey32Id(pipeline_id));
    }
}

#[derive(Resource)]
pub struct PrefixSumFirstPass {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for PrefixSumFirstPass {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
        }
    }
}

impl SpecializedComputePipeline for PrefixSumFirstPass {
    type Key = SolariPathTracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("prefix_sum_first_pass".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SORT_RAYS_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "prefix_sum_step1".into(),
        }
    }
}

#[derive(Component)]
pub struct PrefixSumFirstPassId(pub CachedComputePipelineId);

pub fn prepare_pipelines5(
    views: Query<Entity, With<SolariPathTracer>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<PrefixSumFirstPass>>,
    pipeline: Res<PrefixSumFirstPass>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariPathTracerPipelineKey {});

        commands
            .entity(entity)
            .insert(PrefixSumFirstPassId(pipeline_id));
    }
}

#[derive(Resource)]
pub struct PrefixSumSecondPass {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for PrefixSumSecondPass {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
        }
    }
}

impl SpecializedComputePipeline for PrefixSumSecondPass {
    type Key = SolariPathTracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("prefix_sum_second_pass".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SORT_RAYS_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "prefix_sum_block_sum".into(),
        }
    }
}

#[derive(Component)]
pub struct PrefixSumSecondPassId(pub CachedComputePipelineId);

pub fn prepare_pipelines6(
    views: Query<Entity, With<SolariPathTracer>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<PrefixSumSecondPass>>,
    pipeline: Res<PrefixSumSecondPass>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariPathTracerPipelineKey {});

        commands
            .entity(entity)
            .insert(PrefixSumSecondPassId(pipeline_id));
    }
}

#[derive(Resource)]
pub struct PrefixSumThirdPass {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for PrefixSumThirdPass {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
        }
    }
}

impl SpecializedComputePipeline for PrefixSumThirdPass {
    type Key = SolariPathTracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("prefix_sum_third_pass".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SORT_RAYS_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "prefix_sum_step2".into(),
        }
    }
}

#[derive(Component)]
pub struct PrefixSumThirdPassId(pub CachedComputePipelineId);

pub fn prepare_pipelines7(
    views: Query<Entity, With<SolariPathTracer>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<PrefixSumThirdPass>>,
    pipeline: Res<PrefixSumThirdPass>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariPathTracerPipelineKey {});

        commands
            .entity(entity)
            .insert(PrefixSumThirdPassId(pipeline_id));
    }
}

#[derive(Resource)]
pub struct MapArrayKey32Pass {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for MapArrayKey32Pass {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
        }
    }
}

impl SpecializedComputePipeline for MapArrayKey32Pass {
    type Key = SolariPathTracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("map_array_key32".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SORT_RAYS_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "map_array_key32".into(),
        }
    }
}

#[derive(Component)]
pub struct MapArrayKey32PassId(pub CachedComputePipelineId);

pub fn prepare_pipelines8(
    views: Query<Entity, With<SolariPathTracer>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<MapArrayKey32Pass>>,
    pipeline: Res<MapArrayKey32Pass>,
) {
    for entity in &views {
        let pipeline_id =
            pipelines.specialize(&pipeline_cache, &pipeline, SolariPathTracerPipelineKey {});

        commands
            .entity(entity)
            .insert(MapArrayKey32PassId(pipeline_id));
    }
}
