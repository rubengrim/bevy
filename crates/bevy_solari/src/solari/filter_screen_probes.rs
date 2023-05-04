use super::SOLARI_FILTER_SCREEN_PROBES_SHADER;
use crate::{scene::bind_group_layout::SolariSceneResources, SolariSettings};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_render::{
    globals::GlobalsUniform, render_resource::*, renderer::RenderDevice, view::ViewUniform,
};
use std::num::NonZeroU64;

#[derive(Resource)]
pub struct SolariFilterScreenProbesPipeline {
    pub bind_group_layout: BindGroupLayout,
    scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariFilterScreenProbesPipeline {
    fn from_world(world: &mut World) -> Self {
        let scene_resources = world.resource::<SolariSceneResources>();
        let render_device = world.resource::<RenderDevice>();

        Self {
            bind_group_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("solari_filter_screen_probes_bind_group_layout"),
                entries: &[
                    // View
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
                    // Globals
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(GlobalsUniform::min_size()),
                        },
                        count: None,
                    },
                    // G-buffer
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::Rgba16Uint,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // M-buffer
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::Rgba16Uint,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // Screen probe spherical harmonics
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(112) }),
                        },
                        count: None,
                    },
                    // View target
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
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
pub struct SolariFilterScreenProbesPipelineKey {}

impl SpecializedComputePipeline for SolariFilterScreenProbesPipeline {
    type Key = SolariFilterScreenProbesPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("solari_filter_screen_probes_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: SOLARI_FILTER_SCREEN_PROBES_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "filter_screen_probes".into(),
        }
    }
}

#[derive(Component)]
pub struct SolariFilterScreenProbesPipelineId(pub CachedComputePipelineId);

pub fn prepare_filter_screen_probe_pipelines(
    views: Query<Entity, With<SolariSettings>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<SolariFilterScreenProbesPipeline>>,
    pipeline: Res<SolariFilterScreenProbesPipeline>,
) {
    for entity in &views {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            SolariFilterScreenProbesPipelineKey {},
        );

        commands
            .entity(entity)
            .insert(SolariFilterScreenProbesPipelineId(pipeline_id));
    }
}
