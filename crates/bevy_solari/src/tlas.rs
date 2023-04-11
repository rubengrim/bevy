use crate::blas::BlasStorage;
use bevy_asset::Handle;
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use bevy_render::{
    prelude::Mesh,
    render_resource::{raytrace::*, CommandEncoderDescriptor},
    renderer::{RenderDevice, RenderQueue},
};
use bevy_transform::prelude::GlobalTransform;

#[derive(Resource, Default)]
pub struct TlasResource(pub Option<Tlas>);

pub fn prepare_tlas(
    meshes: Query<(&Handle<Mesh>, &GlobalTransform)>,
    blas_storage: Res<BlasStorage>,
    mut tlas_resource: ResMut<TlasResource>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let meshes = meshes
        .iter()
        .filter_map(|(mesh, transform)| {
            blas_storage.get(mesh).map(|blas| {
                (
                    blas,
                    transform.compute_matrix().transpose().to_cols_array()[..12]
                        .try_into()
                        .unwrap(),
                )
            })
        })
        .collect::<Vec<_>>();

    if meshes.is_empty() {
        tlas_resource.0 = None;
        return;
    }

    let tlas = render_device
        .wgpu_device()
        .create_tlas(&CreateTlasDescriptor {
            label: Some("tlas"),
            flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: AccelerationStructureUpdateMode::Build,
            max_instances: meshes.len() as u32,
        });

    let mut tlas_package = TlasPackage::new(tlas, meshes.len() as u32);
    for (i, (blas, transform)) in meshes.into_iter().enumerate() {
        *tlas_package.get_mut_single(i).unwrap() =
            Some(TlasInstance::new(blas, transform, i as u32, 0xff));
    }

    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("prepare_tlas_command_encoder"),
    });
    command_encoder.build_acceleration_structures(&[], &[tlas_package]);
    render_queue.submit([command_encoder.finish()]);

    // TODO
    // tlas_resource.0 = Some(tlas);
}
