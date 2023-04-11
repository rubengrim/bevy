use crate::tlas::TlasResource;
use bevy_asset::Handle;
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, Resource},
    world::{FromWorld, World},
};
use bevy_render::{
    prelude::Mesh,
    render_resource::*,
    renderer::RenderDevice,
    view::{ExtractedView, ViewUniform, ViewUniforms},
    Extract,
};
use bevy_transform::prelude::GlobalTransform;

pub fn extract_transforms(
    meshes: Extract<Query<(Entity, &GlobalTransform), With<Handle<Mesh>>>>,
    mut commands: Commands,
) {
    commands.insert_or_spawn_batch(
        meshes
            .iter()
            .map(|(entity, transform)| (entity, transform.clone()))
            .collect::<Vec<_>>(),
    );
}

#[derive(Resource)]
pub struct SolariPipeline {
    view_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SolariPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let view_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("view_bind_group_layout"),
                entries: &[
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
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::AccelerationStructure,
                        count: None,
                    },
                ],
            });

        Self {
            view_bind_group_layout,
        }
    }
}

#[derive(Component)]
pub struct ViewBindGroup(pub BindGroup);

pub fn queue_view_bind_group(
    views: Query<Entity, With<ExtractedView>>,
    view_uniforms: Res<ViewUniforms>,
    tlas: Res<TlasResource>,
    pipeline: Res<SolariPipeline>,
    render_device: Res<RenderDevice>,
    mut commands: Commands,
) {
    if let (Some(view_uniforms), Some(tlas)) = (view_uniforms.uniforms.binding(), &tlas.0) {
        let create_view_bind_group = |entity| {
            (
                entity,
                ViewBindGroup(render_device.create_bind_group(&BindGroupDescriptor {
                    label: Some("view_bind_group"),
                    layout: &pipeline.view_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: view_uniforms.clone(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: tlas.as_binding(),
                        },
                    ],
                })),
            )
        };

        commands
            .insert_or_spawn_batch(views.iter().map(create_view_bind_group).collect::<Vec<_>>());
    }
}
