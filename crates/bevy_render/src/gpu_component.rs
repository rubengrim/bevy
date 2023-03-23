use crate::{
    render_resource::{DynamicUniformBuffer, GpuBuffer, GpuBufferable},
    renderer::{RenderDevice, RenderQueue},
    Render, RenderApp, RenderSet,
};
use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::{Component, Entity},
    schedule::IntoSystemConfigs,
    system::{Commands, Query, Res, ResMut, Resource},
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// This plugin prepares the components of the corresponding type for the GPU
/// by transforming them into uniforms.
///
/// They can then be accessed from the [`ComponentUniforms`] resource.
/// For referencing the newly created uniforms a [`DynamicUniformIndex`] is inserted
/// for every processed entity.
///
/// Therefore it sets up the [`RenderSet::Prepare`](crate::RenderSet::Prepare) step
/// for the specified [`ExtractComponent`].
pub struct GpuUniformComponentPlugin<C: Component + GpuBufferable>(PhantomData<C>);

impl<C: Component + GpuBufferable> Plugin for GpuUniformComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .insert_resource(GpuComponentUniforms::<C>::default())
                .add_systems(
                    Render,
                    prepare_uniform_components::<C>.in_set(RenderSet::Prepare),
                );
        }
    }
}

impl<C: Component + GpuBufferable> Default for GpuUniformComponentPlugin<C> {
    fn default() -> Self {
        Self(PhantomData::<C>)
    }
}

#[derive(Resource)]
pub struct GpuComponentUniforms<C: Component + GpuBufferable>(DynamicUniformBuffer<C>);

impl<C: Component + GpuBufferable> Deref for GpuComponentUniforms<C> {
    type Target = DynamicUniformBuffer<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Component + GpuBufferable> DerefMut for GpuComponentUniforms<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<C: Component + GpuBufferable> Default for GpuComponentUniforms<C> {
    fn default() -> Self {
        Self(DynamicUniformBuffer::default())
    }
}

#[derive(Component)]
pub struct GpuComponentUniformOffset<C: Component + GpuBufferable>(pub u32, PhantomData<C>);

/// This system prepares all components of the corresponding component type.
/// They are transformed into uniforms and stored in the [`ComponentUniforms`] resource.
fn prepare_uniform_components<C: Component + GpuBufferable>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut uniforms: ResMut<GpuComponentUniforms<C>>,
    components: Query<(Entity, &C)>,
) {
    uniforms.clear();

    let entities = components
        .iter()
        .map(|(entity, component)| {
            (
                entity,
                GpuComponentUniformOffset(uniforms.push(component.clone()), PhantomData::<C>),
            )
        })
        .collect::<Vec<_>>();
    commands.insert_or_spawn_batch(entities);

    uniforms.write_buffer(&render_device, &render_queue);
}

// ----------------------------------------------------------------------------

pub struct GpuBufferComponentPlugin<C: Component + GpuBufferable>(PhantomData<C>);

impl<C: Component + GpuBufferable> Plugin for GpuBufferComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .insert_resource(GpuBuffer::<C>::new(
                    render_app.world.resource::<RenderDevice>(),
                ))
                .add_systems(
                    Render,
                    prepare_buffer_components::<C>.in_set(RenderSet::Prepare),
                );
        }
    }
}

impl<C: Component + GpuBufferable> Default for GpuBufferComponentPlugin<C> {
    fn default() -> Self {
        Self(PhantomData::<C>)
    }
}

fn prepare_buffer_components<C: Component + GpuBufferable>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut gpu_buffer: ResMut<GpuBuffer<C>>,
    components: Query<(Entity, &C)>,
) {
    gpu_buffer.clear();

    let entities = components
        .iter()
        .map(|(entity, component)| (entity, gpu_buffer.push(component.clone())))
        .collect::<Vec<_>>();
    commands.insert_or_spawn_batch(entities);

    gpu_buffer.write_buffer(&render_device, &render_queue);
}
