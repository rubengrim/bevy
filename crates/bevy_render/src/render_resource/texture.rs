use crate::define_atomic_id;
use std::ops::Deref;

use crate::render_resource::resource_macros::*;

define_atomic_id!(TextureId);
render_resource_wrapper!(ErasedTexture, wgpu::Texture);

/// A GPU-accessible texture.
///
/// May be converted from and dereferences to a wgpu [`Texture`](wgpu::Texture).
/// Can be created via [`RenderDevice::create_texture`](crate::renderer::RenderDevice::create_texture).
#[derive(Clone, Debug)]
pub struct Texture {
    id: TextureId,
    value: ErasedTexture,
}

impl Texture {
    /// Returns the [`TextureId`].
    #[inline]
    pub fn id(&self) -> TextureId {
        self.id
    }

    /// Creates a view of this texture.
    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> TextureView {
        TextureView::from(self.value.create_view(desc))
    }
}

impl From<wgpu::Texture> for Texture {
    fn from(value: wgpu::Texture) -> Self {
        Texture {
            id: TextureId::new(),
            value: ErasedTexture::new(value),
        }
    }
}

impl Deref for Texture {
    type Target = wgpu::Texture;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

define_atomic_id!(TextureViewId);
render_resource_wrapper!(ErasedTextureView, wgpu::TextureView);
render_resource_wrapper!(ErasedSurfaceTexture, wgpu::SurfaceTexture);

/// Describes a [`Texture`] with its associated metadata required by a pipeline or [`BindGroup`](super::BindGroup).
#[derive(Clone, Debug)]
pub struct TextureView {
    id: TextureViewId,
    value: ErasedTextureView,
}

pub struct SurfaceTexture {
    value: ErasedSurfaceTexture,
}

impl SurfaceTexture {
    pub fn try_unwrap(self) -> Option<wgpu::SurfaceTexture> {
        self.value.try_unwrap()
    }
}

impl TextureView {
    /// Returns the [`TextureViewId`].
    #[inline]
    pub fn id(&self) -> TextureViewId {
        self.id
    }
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        TextureView {
            id: TextureViewId::new(),
            value: ErasedTextureView::new(value),
        }
    }
}

impl From<wgpu::SurfaceTexture> for SurfaceTexture {
    fn from(value: wgpu::SurfaceTexture) -> Self {
        SurfaceTexture {
            value: ErasedSurfaceTexture::new(value),
        }
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Deref for SurfaceTexture {
    type Target = wgpu::SurfaceTexture;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

define_atomic_id!(SamplerId);
render_resource_wrapper!(ErasedSampler, wgpu::Sampler);

/// A Sampler defines how a pipeline will sample from a [`TextureView`].
/// They define image filters (including anisotropy) and address (wrapping) modes, among other things.
///
/// May be converted from and dereferences to a wgpu [`Sampler`](wgpu::Sampler).
/// Can be created via [`RenderDevice::create_sampler`](crate::renderer::RenderDevice::create_sampler).
#[derive(Clone, Debug)]
pub struct Sampler {
    id: SamplerId,
    value: ErasedSampler,
}

impl Sampler {
    /// Returns the [`SamplerId`].
    #[inline]
    pub fn id(&self) -> SamplerId {
        self.id
    }
}

impl From<wgpu::Sampler> for Sampler {
    fn from(value: wgpu::Sampler) -> Self {
        Sampler {
            id: SamplerId::new(),
            value: ErasedSampler::new(value),
        }
    }
}

impl Deref for Sampler {
    type Target = wgpu::Sampler;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct TextureDescriptorOwned {
    /// Debug label of the texture. This will show up in graphics debuggers for easy identification.
    pub label: String,
    /// Size of the texture. All components must be greater than zero. For a
    /// regular 1D/2D texture, the unused sizes will be 1. For 2DArray textures,
    /// Z is the number of 2D textures in that array.
    pub size: wgpu::Extent3d,
    /// Mip count of texture. For a texture with no extra mips, this must be 1.
    pub mip_level_count: u32,
    /// Sample count of texture. If this is not 1, texture must have [`BindingType::Texture::multisampled`] set to true.
    pub sample_count: u32,
    /// Dimensions of the texture.
    pub dimension: wgpu::TextureDimension,
    /// Format of the texture.
    pub format: wgpu::TextureFormat,
    /// Allowed usages of the texture. If used in other ways, the operation will panic.
    pub usage: wgpu::TextureUsages,
    /// Specifies what view formats will be allowed when calling create_view() on this texture.
    ///
    /// View formats of the same format as the texture are always allowed.
    ///
    /// Note: currently, only the srgb-ness is allowed to change. (ex: Rgba8Unorm texture + Rgba8UnormSrgb view)
    pub view_formats: &'static [wgpu::TextureFormat],
}
