use super::{camera::SolariSettings, update_screen_probes::SolariUpdateScreenProbesPipeline};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use bevy_render::{
    camera::ExtractedCamera,
    render_resource::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d,
        TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    renderer::RenderDevice,
    texture::{CachedTexture, TextureCache},
    view::ViewUniforms,
};

#[derive(Component)]
pub struct SolariResources {
    screen_probes: CachedTexture,
}

pub fn prepare_resources(
    views: Query<(Entity, &ExtractedCamera), With<SolariSettings>>,
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
) {
    for (entity, camera) in &views {
        if let Some(viewport) = camera.physical_viewport_size {
            let screen_probes = TextureDescriptor {
                label: Some("solari_screen_probes"),
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: round_up_to_multiple_of_8(viewport.x),
                    height: round_up_to_multiple_of_8(viewport.y),
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            commands.entity(entity).insert(SolariResources {
                screen_probes: texture_cache.get(&render_device, screen_probes),
            });
        }
    }
}

#[derive(Component)]
pub struct SolariBindGroups {
    pub update_screen_probes: BindGroup,
}

pub fn queue_bind_groups(
    views: Query<(Entity, &SolariResources)>,
    view_uniforms: Res<ViewUniforms>,
    update_screen_probes_pipeline: Res<SolariUpdateScreenProbesPipeline>,
    mut commands: Commands,
    render_device: Res<RenderDevice>,
) {
    if let Some(view_uniforms) = view_uniforms.uniforms.binding() {
        for (entity, solari_resources) in &views {
            let update_screen_probes = BindGroupDescriptor {
                label: Some("solari_update_screen_probes_bind_group"),
                layout: &update_screen_probes_pipeline.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: view_uniforms.clone(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(
                            &solari_resources.screen_probes.default_view,
                        ),
                    },
                ],
            };

            commands.entity(entity).insert(SolariBindGroups {
                update_screen_probes: render_device.create_bind_group(&update_screen_probes),
            });
        }
    }
}

fn round_up_to_multiple_of_8(x: u32) -> u32 {
    (x + 7) & !7
}
