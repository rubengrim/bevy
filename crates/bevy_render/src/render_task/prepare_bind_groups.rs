use crate::{
    camera::ExtractedCamera,
    render_resource::BindGroupLayout,
    render_task::{
        prepare_pipelines::RenderTaskPipelines, RenderTask, RenderTaskResource,
        RenderTaskResourceRegistry, RenderTaskResourceView,
    },
    renderer::RenderDevice,
};
use bevy_core::FrameCount;
use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Local, Query, Res, ResMut},
};
use bevy_math::UVec2;
use bevy_utils::HashMap;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, SamplerBindingType, ShaderStages, StorageTextureAccess,
    TextureViewDimension,
};

pub fn create_bind_group_layouts<R: RenderTask>(
    render_device: &RenderDevice,
) -> HashMap<&'static str, BindGroupLayout> {
    let task_name = R::name();
    let mut layouts = HashMap::new();
    let resources = R::resources(UVec2::ZERO);

    for (pass_name, pass) in R::passes() {
        let mut entries = Vec::new();
        for (i, resource_view) in pass.bindings.iter().enumerate() {
            entries.push(BindGroupLayoutEntry {
                binding: i as u32,
                visibility: ShaderStages::COMPUTE,
                ty: match resource_view {
                    RenderTaskResourceView::SampledTexture { name, .. } => {
                        let RenderTaskResource::Texture { format, .. } =
                            resources.get(name).unwrap();
                        BindingType::Texture {
                            sample_type: format.sample_type(None).unwrap(),
                            view_dimension: TextureViewDimension::D2, // TODO: Don't hardcode this
                            multisampled: false,
                        }
                    }
                    RenderTaskResourceView::StorageTextureWrite { name, .. } => {
                        let RenderTaskResource::Texture { format, .. } =
                            resources.get(name).unwrap();
                        BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: *format,
                            view_dimension: TextureViewDimension::D2, // TODO: Don't hardcode this
                        }
                    }
                    RenderTaskResourceView::StorageTextureReadWrite { name, .. } => {
                        let RenderTaskResource::Texture { format, .. } =
                            resources.get(name).unwrap();
                        BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: *format,
                            view_dimension: TextureViewDimension::D2, // TODO: Don't hardcode this
                        }
                    }
                    RenderTaskResourceView::Sampler(_) => {
                        // TODO: Don't hardcode filtering
                        BindingType::Sampler(SamplerBindingType::Filtering)
                    }
                },
                count: None,
            });
        }

        layouts.insert(
            pass_name,
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some(&format!("{task_name}_{pass_name}_bind_group_layout")),
                entries: &entries,
            }),
        );
    }

    layouts
}

pub fn prepare_bind_groups<R: RenderTask>(
    query: Query<(Entity, &ExtractedCamera), With<R::RenderTaskSettings>>,
    mut resource_registry: ResMut<RenderTaskResourceRegistry>,
    mut previous_viewport_sizes: Local<HashMap<Entity, UVec2>>,
    pipelines: Res<RenderTaskPipelines<R>>,
    frame_count: Res<FrameCount>,
    render_device: Res<RenderDevice>,
) {
    let task_name = R::name();

    for (entity, camera) in &query {
        // Skip creating bind groups for views with the same viewport as last frame
        let Some(physical_viewport_size) = camera.physical_viewport_size else { continue };
        match previous_viewport_sizes.get(&entity) {
            Some(previous_viewport_size) if *previous_viewport_size == physical_viewport_size => {
                continue;
            }
            _ => {
                previous_viewport_sizes.insert(entity, physical_viewport_size);
            }
        }

        for (pass_name, pass) in R::passes() {
            let mut entries = Vec::new();
            for (i, resource_view) in pass.bindings.iter().enumerate() {
                entries.push(BindGroupEntry {
                    binding: i as u32,
                    resource: match resource_view {
                        RenderTaskResourceView::SampledTexture {
                            name,
                            mip,
                            layer,
                            previous_frame,
                        } => BindingResource::TextureView(todo!()),
                        RenderTaskResourceView::StorageTextureWrite {
                            name,
                            mip,
                            layer,
                            previous_frame,
                        } => BindingResource::TextureView(todo!()),
                        RenderTaskResourceView::StorageTextureReadWrite {
                            name,
                            mip,
                            layer,
                            previous_frame,
                        } => BindingResource::TextureView(todo!()),
                        RenderTaskResourceView::Sampler(name) => BindingResource::Sampler(todo!()),
                    },
                });
            }

            let descriptor = BindGroupDescriptor {
                label: Some(&format!("{task_name}_{pass_name}_bind_group")),
                layout: pipelines.bind_group_layouts.get(pass_name).unwrap(),
                entries: &entries,
            };
            // TODO: Store/cache bind groups in RenderTaskResourceRegistry
        }
    }
}
