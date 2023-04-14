use crate::{blas::BlasStorage, material::MaterialIndex};
use bevy_asset::Handle;
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use bevy_render::{
    prelude::Mesh,
    render_resource::{raytrace::*, CommandBuffer, CommandEncoderDescriptor},
    renderer::RenderDevice,
};
use bevy_transform::prelude::GlobalTransform;
use once_cell::sync::OnceCell;
use std::iter;

#[derive(Resource, Default)]
pub struct TlasResource(pub Option<TlasPackage>);

pub static mut TLAS_BUILD_COMMAND_BUFFER: OnceCell<CommandBuffer> = OnceCell::new();

pub fn prepare_tlas(
    meshes: Query<(&Handle<Mesh>, &GlobalTransform, &MaterialIndex)>,
    blas_storage: Res<BlasStorage>,
    mut tlas_resource: ResMut<TlasResource>,
    render_device: Res<RenderDevice>,
) {
    // Get BLAS and transform data for each mesh
    let meshes = meshes
        .iter()
        .filter_map(|(mesh, transform, material_index)| {
            blas_storage
                .get(mesh)
                .map(|blas| (blas, map_transform(transform), material_index))
        })
        .collect::<Vec<_>>();

    // Create a TLAS
    let tlas = render_device
        .wgpu_device()
        .create_tlas(&CreateTlasDescriptor {
            label: Some("tlas"),
            flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: AccelerationStructureUpdateMode::Build,
            max_instances: meshes.len() as u32,
        });

    // Fill the TLAS with each mesh instance (BLAS)
    let mut tlas = TlasPackage::new(tlas, meshes.len() as u32);
    for (i, (blas, transform, material_index)) in meshes.into_iter().enumerate() {
        *tlas.get_mut_single(i).unwrap() =
            Some(TlasInstance::new(blas, transform, material_index.0, 0xFF));
    }

    // Build the TLAS
    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("build_tlas_command_encoder"),
    });
    command_encoder.build_acceleration_structures(&[], iter::once(&tlas));
    unsafe {
        TLAS_BUILD_COMMAND_BUFFER
            .set(command_encoder.finish())
            .unwrap();
    }

    tlas_resource.0 = Some(tlas);
}

fn map_transform(transform: &GlobalTransform) -> [f32; 12] {
    transform.compute_matrix().transpose().to_cols_array()[..12]
        .try_into()
        .unwrap()
}
