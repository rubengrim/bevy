use crate::misc::{create_scene_bind_group_layout, create_view_bind_group_layout};
use bevy_asset::HandleUntyped;
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_reflect::TypeUuid;
use bevy_render::{
    render_resource::{
        BindGroupLayout, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, Shader,
        SpecializedComputePipeline, SpecializedComputePipelines,
    },
    renderer::RenderDevice,
    view::ExtractedView,
};

pub const SOLARI_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1717171717171755);

#[derive(Resource)]
pub struct SolariPipeline {
    pub scene_bind_group_layout: BindGroupLayout,
    pub view_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        Self {
            scene_bind_group_layout: create_scene_bind_group_layout(render_device),
            view_bind_group_layout: create_view_bind_group_layout(render_device),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct SolariPipelineKey {}

impl SpecializedComputePipeline for SolariPipeline {
    type Key = SolariPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: "solari_main".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariPipelineId(pub CachedComputePipelineId);

pub fn prepare_pipelines(
    views: Query<Entity, With<ExtractedView>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariPipeline>>,
    pipeline: Res<SolariPipeline>,
) {
    for entity in &views {
        let pipeline_id = pipelines.specialize(&pipeline_cache, &pipeline, SolariPipelineKey {});

        commands
            .entity(entity)
            .insert(SolariPipelineId(pipeline_id));
    }
}
