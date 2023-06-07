use super::{material::GpuSolariMaterial, scene::PreviousGlobalTransform};
use bevy_ecs::{
    system::Resource,
    world::{FromWorld, World},
};
use bevy_math::Mat4;
use bevy_render::{globals::GlobalsUniform, render_resource::*, renderer::RenderDevice};
use bevy_utils::default;
use std::num::NonZeroU32;

#[derive(Resource)]
pub struct SolariSceneResources {
    pub bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
}

impl FromWorld for SolariSceneResources {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        Self {
            bind_group_layout: create_scene_bind_group_layout(render_device),
            sampler: render_device.create_sampler(&SamplerDescriptor {
                mipmap_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                ..default()
            }),
        }
    }
}

fn create_scene_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    let entries = &[
        // TLAS
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::AccelerationStructure,
            count: None,
        },
        // Mesh material indices buffer
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(u32::min_size()),
            },
            count: None,
        },
        // Index buffers
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None, // TODO
            },
            count: Some(unsafe { NonZeroU32::new_unchecked(50_000) }),
        },
        // Vertex buffers
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None, // TODO
            },
            count: Some(unsafe { NonZeroU32::new_unchecked(50_000) }),
        },
        // Transforms buffer
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(Mat4::min_size()),
            },
            count: None,
        },
        // Previous transforms buffer
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(PreviousGlobalTransform::min_size()),
            },
            count: None,
        },
        // Material buffer
        BindGroupLayoutEntry {
            binding: 6,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(GpuSolariMaterial::min_size()),
            },
            count: None,
        },
        // Texture maps
        BindGroupLayoutEntry {
            binding: 7,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: Some(unsafe { NonZeroU32::new_unchecked(50_000) }),
        },
        // Texture sampler
        BindGroupLayoutEntry {
            binding: 8,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        },
        // Emissive object mesh material indices buffer
        BindGroupLayoutEntry {
            binding: 9,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(u32::min_size()),
            },
            count: None,
        },
        // Emissive object triangle counts buffer
        BindGroupLayoutEntry {
            binding: 10,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(u32::min_size()),
            },
            count: None,
        },
        // Globals
        BindGroupLayoutEntry {
            binding: 11,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(GlobalsUniform::min_size()),
            },
            count: None,
        },
    ];

    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("solari_scene_bind_group_layout"),
        entries,
    })
}
