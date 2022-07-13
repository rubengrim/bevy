use crate::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy_app::Plugin;
use bevy_asset::{load_internal_asset, Assets, Handle, HandleUntyped};
use bevy_ecs::prelude::*;
use bevy_reflect::{std_traits::ReflectDefault, Reflect, TypeUuid};
use bevy_render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    mesh::{Mesh, MeshVertexBufferLayout},
    prelude::{Color, Shader},
    render_resource::{
        AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
    },
};

pub const WIREFRAME_MATERIAL_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 192598014480025766);

/// A [`Plugin`] that draws wireframes.
#[derive(Debug, Default)]
pub struct WireframePlugin;
impl Plugin for WireframePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        load_internal_asset!(
            app,
            WIREFRAME_MATERIAL_SHADER_HANDLE,
            "render/wireframe.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(MaterialPlugin::<WireframeMaterial>::default());

        app.register_type::<WireframeConfig>()
            .init_resource::<WireframeConfig>()
            .add_plugin(ExtractResourcePlugin::<WireframeConfig>::default());

        app.add_system(apply_global)
            .add_system(apply_material)
            .add_system(wireframe_color_changed)
            .add_system(global_color_changed);
    }
}

/// Toggles wireframe rendering for any entity it is attached to.
///
/// This requires the [`WireframePlugin`] to be enabled.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component, Default)]
pub struct Wireframe;

/// Sets the color of the [`Wireframe`] of the entity it is attached to.
///
/// This overrides the [`WireframeConfig::default_color`].
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component, Default)]
pub struct WireframeColor {
    pub color: Color,
}

/// Configuration resource for [`WireframePlugin`].
#[derive(Debug, Clone, Default, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct WireframeConfig {
    /// Whether to show wireframes for all meshes.
    /// If `false`, only meshes with a [Wireframe] component will be rendered.
    pub global: bool,
    /// If [`Self::global`] is set, any [`Entity`] that does not have a [`Wireframe`] component attached to it will have
    /// wireframes in this color. Otherwise, this will be the fallback color for any entity that has a [`Wireframe`],
    /// but no [`WireframeColor`].
    pub color: Color,
}

/// Apply the wireframe material to any mesh with a `Wireframe` component.
/// Uses `WireframeConfig::color` as a fallback if no `WireframeColor` component is found
#[allow(clippy::type_complexity)]
fn apply_material(
    mut commands: Commands,
    config: Res<WireframeConfig>,
    mut materials: ResMut<Assets<WireframeMaterial>>,
    wireframes: Query<
        (Entity, Option<&WireframeColor>),
        (With<Wireframe>, Without<Handle<WireframeMaterial>>),
    >,
) {
    for (e, color) in &wireframes {
        commands.entity(e).insert(materials.add(WireframeMaterial {
            color: if let Some(wireframe_color) = color {
                wireframe_color.color
            } else {
                config.color
            },
        }));
    }
}

/// Updates the wireframe material when the color in `WireframeColor` changes
#[allow(clippy::type_complexity)]
fn wireframe_color_changed(
    mut materials: ResMut<Assets<WireframeMaterial>>,
    mut colors_changed: Query<
        (&mut Handle<WireframeMaterial>, &WireframeColor),
        (With<Wireframe>, Changed<WireframeColor>),
    >,
) {
    for (mut handle, wireframe_color) in &mut colors_changed {
        *handle = materials.add(WireframeMaterial {
            color: wireframe_color.color,
        });
    }
}

/// Updates the wireframe material of all entities without a specified color or without a `Wireframe` component
fn global_color_changed(
    config: Res<WireframeConfig>,
    mut materials: ResMut<Assets<WireframeMaterial>>,
    mut wireframes: Query<&mut Handle<WireframeMaterial>, Without<WireframeColor>>,
) {
    if !config.is_changed() {
        return;
    }

    for mut handle in &mut wireframes {
        *handle = materials.add(WireframeMaterial {
            color: config.color,
        });
    }
}

/// Applies or remove a wireframe material on any mesh without a `Wireframe` component.
#[allow(clippy::type_complexity)]
fn apply_global(
    mut commands: Commands,
    config: Res<WireframeConfig>,
    mut materials: ResMut<Assets<WireframeMaterial>>,
    mut q1: ParamSet<(
        Query<
            Entity,
            (
                With<Handle<Mesh>>,
                Without<Handle<WireframeMaterial>>,
                Without<Wireframe>,
            ),
        >,
        Query<
            Entity,
            (
                With<Handle<Mesh>>,
                With<Handle<WireframeMaterial>>,
                Without<Wireframe>,
            ),
        >,
    )>,
    mut is_global_applied: Local<bool>,
) {
    if !config.is_changed() {
        return;
    }

    if !*is_global_applied && config.global {
        let global_material = materials.add(WireframeMaterial {
            color: config.color,
        });

        for e in &mut q1.p0() {
            commands.entity(e).insert(global_material.clone());
        }

        *is_global_applied = true;
    } else if *is_global_applied && !config.global {
        for e in &mut q1.p1() {
            commands.entity(e).remove::<Handle<WireframeMaterial>>();
        }
        *is_global_applied = false;
    }
}

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "9e694f70-9963-4418-8bc1-3474c66b13b8"]
struct WireframeMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for WireframeMaterial {
    fn fragment_shader() -> ShaderRef {
        WIREFRAME_MATERIAL_SHADER_HANDLE.typed().into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        descriptor.depth_stencil.as_mut().unwrap().bias.slope_scale = 1.0;
        Ok(())
    }
}
