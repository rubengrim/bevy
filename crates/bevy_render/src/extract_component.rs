use crate::{view::ComputedVisibility, Extract, ExtractSchedule, RenderApp};
use bevy_app::{App, Plugin};
use bevy_asset::{Asset, Handle};
use bevy_ecs::{
    component::Component,
    prelude::*,
    query::{QueryItem, ReadOnlyWorldQuery, WorldQuery},
    system::lifetimeless::Read,
};
use std::marker::PhantomData;

pub use bevy_render_macros::ExtractComponent;

/// Describes how a component gets extracted for rendering.
///
/// Therefore the component is transferred from the "app world" into the "render world"
/// in the [`ExtractSchedule`](crate::ExtractSchedule) step.
pub trait ExtractComponent: Component {
    /// ECS [`WorldQuery`] to fetch the components to extract.
    type Query: WorldQuery + ReadOnlyWorldQuery;
    /// Filters the entities with additional constraints.
    type Filter: WorldQuery + ReadOnlyWorldQuery;

    /// The output from extraction.
    ///
    /// Returning `None` based on the queried item can allow early optimization,
    /// for example if there is an `enabled: bool` field on `Self`, or by only accepting
    /// values within certain thresholds.
    ///
    /// The output may be different from the queried component.
    /// This can be useful for example if only a subset of the fields are useful
    /// in the render world.
    ///
    /// `Out` has a [`Bundle`] trait bound instead of a [`Component`] trait bound in order to allow use cases
    /// such as tuples of components as output.
    type Out: Bundle;

    // TODO: https://github.com/rust-lang/rust/issues/29661
    // type Out: Component = Self;

    /// Defines how the component is transferred into the "render world".
    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out>;
}

/// This plugin extracts the components into the "render world".
///
/// Therefore it sets up the [`ExtractSchedule`](crate::ExtractSchedule) step
/// for the specified [`ExtractComponent`].
pub struct ExtractComponentPlugin<C, F = ()> {
    only_extract_visible: bool,
    marker: PhantomData<fn() -> (C, F)>,
}

impl<C, F> Default for ExtractComponentPlugin<C, F> {
    fn default() -> Self {
        Self {
            only_extract_visible: false,
            marker: PhantomData,
        }
    }
}

impl<C, F> ExtractComponentPlugin<C, F> {
    pub fn extract_visible() -> Self {
        Self {
            only_extract_visible: true,
            marker: PhantomData,
        }
    }
}

impl<C: ExtractComponent> Plugin for ExtractComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            if self.only_extract_visible {
                render_app.add_systems(ExtractSchedule, extract_visible_components::<C>);
            } else {
                render_app.add_systems(ExtractSchedule, extract_components::<C>);
            }
        }
    }
}

impl<T: Asset> ExtractComponent for Handle<T> {
    type Query = Read<Handle<T>>;
    type Filter = ();
    type Out = Handle<T>;

    #[inline]
    fn extract_component(handle: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(handle.clone_weak())
    }
}

/// This system extracts all components of the corresponding [`ExtractComponent`] type.
fn extract_components<C: ExtractComponent>(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Extract<Query<(Entity, C::Query), C::Filter>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, query_item) in &query {
        if let Some(component) = C::extract_component(query_item) {
            values.push((entity, component));
        }
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

/// This system extracts all visible components of the corresponding [`ExtractComponent`] type.
fn extract_visible_components<C: ExtractComponent>(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Extract<Query<(Entity, &ComputedVisibility, C::Query), C::Filter>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility, query_item) in &query {
        if computed_visibility.is_visible() {
            if let Some(component) = C::extract_component(query_item) {
                values.push((entity, component));
            }
        }
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}
