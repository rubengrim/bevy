//! Run a prepass before the main pass to get the depth and/or normals texture.
//! The depth prepass texture is then used by the main pass to reduce overdraw.
//!
//! To enable the prepass, you need to add a `PrepassSettings` component to a `Camera`.
//! Both textures are available on the `PrepassTextures` component attached to each `Camera` with a `PrepassSettings`
//!
//! Currently only works for 3d

pub mod node;

use bevy_ecs::prelude::*;
use bevy_render::{
    render_phase::{CachedRenderPipelinePhaseItem, DrawFunctionId, EntityPhaseItem, PhaseItem},
    render_resource::{CachedRenderPipelineId, Extent3d},
    texture::CachedTexture,
};
use bevy_utils::FloatOrd;

/// Add a [`PrepassSettings`] component to a [`crate::prelude::Camera3d`] to perform a depth and/or normal prepass.
/// These textures are useful for various screen-space effects and reducing overdraw in the main pass.
///
/// The prepass runs for each `Material`, you can control if the prepass should run by setting the `prepass_enabled`
/// flag on the `MaterialPlugin`.
///
/// The textures are automatically added to the default mesh view bindings. You can also get the raw textures
/// by querying the [`ViewPrepassTextures`] component on the camera with the [`PrepassSettings`].
///
/// When using the default mesh view bindings you should be able to use `prepass_depth()`
/// and `prepass_normal()` to load the related textures. These functions are defined in `bevy_pbr::utils`.
/// See the `shader_prepass` example that shows how to use it.
#[derive(Clone, Component)]
pub struct PrepassSettings {
    /// If true then depth values will be copied to a separate texture available to the main pass.
    /// The main pass already uses a depth texture by default which helps reduce overdraw, but this will help reduce it even more.
    ///
    /// Make sure to enable the prepass on your `Material` for this to do anything.
    pub depth_enabled: bool,
    /// If true then vertex world normals will be copied to a separate texture available to the main pass.
    ///
    /// Make sure to enable the prepass on your `Material` for this to do anything.
    pub normal_enabled: bool,
}

impl Default for PrepassSettings {
    fn default() -> Self {
        Self {
            depth_enabled: true,
            normal_enabled: false,
        }
    }
}

/// Textures that are written to by the prepass.
///
/// This component only exists if any of the relevant options on [`PrepassSettings`] are `true`, and the prepass is enabled.
#[derive(Component)]
pub struct ViewPrepassTextures {
    /// The depth texture generated by the prepass.
    /// Exists only if `depth_enabled` on [`PrepassSettings`] is true.
    pub depth: Option<CachedTexture>,
    /// The normals texture generated by the prepass.
    /// Exists only if `normal_enabled` on [`PrepassSettings`] is true.
    pub normal: Option<CachedTexture>,
    /// The size of the textures.
    pub size: Extent3d,
}

/// Opaque phase of the 3d prepass.
///
/// Sorted on the distance.
///
/// Used to render all 3d meshes with materials that have no transparency.
pub struct Opaque3dPrepass {
    pub distance: f32,
    pub entity: Entity,
    pub pipeline_id: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
}

impl PhaseItem for Opaque3dPrepass {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        FloatOrd(self.distance)
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    #[inline]
    fn sort(items: &mut [Self]) {
        radsort::sort_by_key(items, |item| item.distance);
    }
}

impl EntityPhaseItem for Opaque3dPrepass {
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl CachedRenderPipelinePhaseItem for Opaque3dPrepass {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline_id
    }
}

/// Alpha mask phase of the 3d prepaas.
///
/// Sorted on the distance.
///
/// Used to render all meshes with a material with an alpha mask.
pub struct AlphaMask3dPrepass {
    pub distance: f32,
    pub entity: Entity,
    pub pipeline_id: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
}

impl PhaseItem for AlphaMask3dPrepass {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        FloatOrd(self.distance)
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    #[inline]
    fn sort(items: &mut [Self]) {
        radsort::sort_by_key(items, |item| item.distance);
    }
}

impl EntityPhaseItem for AlphaMask3dPrepass {
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl CachedRenderPipelinePhaseItem for AlphaMask3dPrepass {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline_id
    }
}
