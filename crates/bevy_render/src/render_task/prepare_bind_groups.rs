use super::RenderTask;
use crate::{
    render_resource::BindGroupLayout,
    render_task::{RenderTaskResource, RenderTaskResourceView},
    renderer::RenderDevice,
};
use bevy_math::UVec2;
use bevy_utils::HashMap;
use wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, SamplerBindingType, ShaderStages,
    StorageTextureAccess, TextureViewDimension,
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

pub fn prepare_bind_groups<R: RenderTask>() {
    // TODO
}
