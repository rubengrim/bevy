use super::SOLARI_UPDATE_SCREEN_PROBES_SHADER;
use crate::{scene::bind_group_layout::SolariSceneResources, SolariSettings};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::{render_resource::*, renderer::RenderDevice, view::ViewUniform};

#[derive(Resource)]
pub struct SolariUpdateScreenProbesPipeline {
    pub bind_group_layout: BindGroupLayout,
    scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariUpdateScreenProbesPipeline {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        let render_device = world.resource::<RenderDevice>();

        Self {
            bind_group_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("solari_update_screen_probes_bind_group_layout"),
                entries: &[
                    // View uniforms
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(ViewUniform::min_size()),
                        },
                        count: None,
                    },
                    // Screen probes
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            }),
            scene_bind_group_layout: scene_resources.bind_group_layout.clone(),
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
