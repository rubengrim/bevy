#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(rustdoc::all)]
#[allow(unused)]
mod fsr {
    type VkDevice = ash::vk::Device;
    type VkPhysicalDevice = ash::vk::PhysicalDevice;
    type PFN_vkGetDeviceProcAddr = ash::vk::PFN_vkGetDeviceProcAddr;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use bevy_math::{UVec2, Vec2};
use bevy_render::{
    prelude::PerspectiveProjection,
    render_resource::CommandEncoder,
    renderer::{
        wgpu_hal_api::{Api, Vulkan},
        RenderDevice,
    },
    texture::GpuImage,
};
use fsr::{
    ffxFsr2ContextCreate, ffxFsr2ContextDestroy, ffxFsr2ContextDispatch, ffxFsr2GetJitterOffset,
    ffxFsr2GetJitterPhaseCount, ffxGetDeviceVK, ffxGetInterfaceVK, ffxGetResourceVK,
    ffxGetScratchMemorySizeVK, FfxDimensions2D, FfxErrorCodes_FFX_OK, FfxFloatCoords2D,
    FfxFsr2Context, FfxFsr2ContextDescription, FfxFsr2DispatchDescription,
    FfxFsr2InitializationFlagBits, FfxInterface, FfxResource, FfxResourceDescription,
    FfxResourceDescription__bindgen_ty_1, FfxResourceDescription__bindgen_ty_2,
    FfxResourceDescription__bindgen_ty_3, FfxResourceFlags_FFX_RESOURCE_FLAGS_NONE,
    FfxResourceStates_FFX_RESOURCE_STATE_PIXEL_COMPUTE_READ,
    FfxResourceType_FFX_RESOURCE_TYPE_TEXTURE2D, FfxSurfaceFormat, VkDeviceContext,
};
use std::{mem::MaybeUninit, ops::Sub, ptr};

// TODO
const MAX_CONTEXTS: usize = 1;

pub struct FsrInterface {
    interface: FfxInterface,
    scratch_memory: Box<[u8]>,
    render_device: RenderDevice,
}

impl FsrInterface {
    pub fn new(render_device: RenderDevice) -> Self {
        let c = |device: Option<&<Vulkan as Api>::Device>| unsafe {
            let device = device.unwrap(); // TODO: Error if not Vulkan
            let get_device_proc_addr = device
                .shared_instance()
                .raw_instance()
                .fp_v1_0()
                .get_device_proc_addr;
            let physical_device = device.raw_physical_device();
            let device = device.raw_device().handle();

            let scratch_memory_size = ffxGetScratchMemorySizeVK(physical_device, MAX_CONTEXTS);
            let mut scratch_memory = Vec::with_capacity(scratch_memory_size).into_boxed_slice();

            let mut interface = MaybeUninit::uninit();
            let return_code = ffxGetInterfaceVK(
                interface.as_mut_ptr(),
                ffxGetDeviceVK(&mut VkDeviceContext {
                    vkDevice: device,
                    vkPhysicalDevice: physical_device,
                    vkDeviceProcAddr: get_device_proc_addr,
                }),
                scratch_memory.as_mut_ptr() as *mut _,
                scratch_memory_size,
                MAX_CONTEXTS,
            );
            assert_eq!(return_code, FfxErrorCodes_FFX_OK);

            Self {
                interface: interface.assume_init(),
                scratch_memory,
                render_device: render_device.clone(),
            }
        };

        unsafe { render_device.wgpu_device().as_hal::<Vulkan, _, _>(c) }
    }

    pub fn create_context(
        &mut self,
        max_render_size: UVec2,
        presentation_size: UVec2,
        hdr: bool,
        dynamic_resolution_scaling: bool,
    ) -> FsrContext {
        let mut flags = FfxFsr2InitializationFlagBits::FFX_FSR2_ENABLE_DEPTH_INVERTED
            | FfxFsr2InitializationFlagBits::FFX_FSR2_ENABLE_DEPTH_INFINITE
            | FfxFsr2InitializationFlagBits::FFX_FSR2_ENABLE_AUTO_EXPOSURE;
        if hdr {
            flags |= FfxFsr2InitializationFlagBits::FFX_FSR2_ENABLE_HIGH_DYNAMIC_RANGE;
        }
        if dynamic_resolution_scaling {
            flags |= FfxFsr2InitializationFlagBits::FFX_FSR2_ENABLE_DYNAMIC_RESOLUTION;
        }
        if cfg!(debug_assertions) {
            flags |= FfxFsr2InitializationFlagBits::FFX_FSR2_ENABLE_DEBUG_CHECKING;
        }

        let context_description = FfxFsr2ContextDescription {
            flags: flags.0 as _, // TODO: Is this correct?
            maxRenderSize: FfxDimensions2D {
                width: max_render_size.x,
                height: max_render_size.y,
            },
            displaySize: FfxDimensions2D {
                width: presentation_size.x,
                height: presentation_size.y,
            },
            backendInterface: self.interface,
            fpMessage: None, // TODO
        };

        let mut context = MaybeUninit::uninit();
        let context = unsafe {
            let return_code = ffxFsr2ContextCreate(context.as_mut_ptr(), &context_description);
            assert_eq!(return_code, FfxErrorCodes_FFX_OK);
            context.assume_init()
        };

        FsrContext {
            presentation_size,
            context,
            render_device: self.render_device.clone(),
        }
    }
}

pub struct FsrContext {
    presentation_size: UVec2,
    context: FfxFsr2Context,
    render_device: RenderDevice,
}

impl FsrContext {
    pub fn suggested_jitter(&self, render_size: UVec2, frame_count: u32) -> Vec2 {
        let phase_count = unsafe {
            ffxFsr2GetJitterPhaseCount(render_size.x as i32, self.presentation_size.x as i32)
        };
        let mut jitter = Vec2::ZERO;
        unsafe {
            let return_code = ffxFsr2GetJitterOffset(
                &mut jitter.x,
                &mut jitter.y,
                frame_count as i32 % phase_count,
                phase_count,
            );
            assert_eq!(return_code, FfxErrorCodes_FFX_OK);
        }
        jitter
    }

    pub fn suggested_mip_bias(&self, render_size: UVec2) -> f32 {
        (render_size.x as f32 / self.presentation_size.x as f32)
            .sub(1.0)
            .log2()
    }

    pub fn dispatch<'a>(&mut self, dispatch_description: FsrDispatchDescription<'a>) {
        let dispatch_description = FfxFsr2DispatchDescription {
            commandList: todo!(),
            color: todo!(),
            depth: todo!(),
            motionVectors: todo!(),
            exposure: todo!(),
            reactive: todo!(),
            transparencyAndComposition: todo!(),
            output: todo!(),
            jitterOffset: FfxFloatCoords2D {
                x: dispatch_description.jitter.x,
                y: dispatch_description.jitter.y,
            },
            motionVectorScale: FfxFloatCoords2D {
                x: dispatch_description.render_size.x as f32,
                y: dispatch_description.render_size.y as f32,
            },
            renderSize: FfxDimensions2D {
                width: dispatch_description.render_size.x,
                height: dispatch_description.render_size.y,
            },
            enableSharpening: false,
            sharpness: 0.0,
            frameTimeDelta: dispatch_description.frame_time_delta,
            preExposure: 0.0,
            reset: dispatch_description.reset,
            cameraNear: dispatch_description.camera_projection.near,
            cameraFar: dispatch_description.camera_projection.far, // TODO: f32::INFINITY instead?
            cameraFovAngleVertical: dispatch_description.camera_projection.fov,
            viewSpaceToMetersFactor: todo!(),
            enableAutoReactive: false,
            colorOpaqueOnly: todo!(),
            autoTcThreshold: 0.0,
            autoTcScale: 0.0,
            autoReactiveScale: 0.0,
            autoReactiveMax: 0.0,
        };

        unsafe {
            let return_code = ffxFsr2ContextDispatch(&mut self.context, &dispatch_description);
            assert_eq!(return_code, FfxErrorCodes_FFX_OK);
        }
    }
}

impl Drop for FsrContext {
    fn drop(&mut self) {
        let c = |device: Option<&<Vulkan as Api>::Device>| unsafe {
            device
                .unwrap() // TODO: Error if not Vulkan
                .raw_device()
                .device_wait_idle()
                .expect("Failed to wait for idle device when destroying FsrContext");

            let return_code = ffxFsr2ContextDestroy(&mut self.context);
            assert_eq!(return_code, FfxErrorCodes_FFX_OK);
        };

        unsafe { self.render_device.wgpu_device().as_hal::<Vulkan, _, _>(c) };
    }
}

pub struct FsrDispatchDescription<'a> {
    pub command_encoder: &'a mut CommandEncoder,
    pub color_and_output: &'a GpuImage,
    pub depth: &'a GpuImage,
    pub motion_vectors: &'a GpuImage,
    pub jitter: Vec2,
    pub render_size: UVec2,
    pub frame_time_delta: f32,
    pub reset: bool,
    pub camera_projection: &'a PerspectiveProjection,
}

// TODO: Lots of cleanup and double checking I did things right needed
fn ffx_texture(image: &GpuImage) -> FfxResource {
    unsafe {
        ffxGetResourceVK(
            // image
            //     .texture
            //     .as_hal::<Vulkan, _, _>(|texture| texture.unwrap().raw_handle()),
            todo!("Need to modify wgpu to allow Texture::as_hal() to return a"),
            FfxResourceDescription {
                type_: FfxResourceType_FFX_RESOURCE_TYPE_TEXTURE2D,
                format: match image.texture_format {
                    _ => todo!(),
                },
                __bindgen_anon_1: FfxResourceDescription__bindgen_ty_1 {
                    width: image.size.x as u32,
                },
                __bindgen_anon_2: FfxResourceDescription__bindgen_ty_2 {
                    height: image.size.y as u32,
                },
                __bindgen_anon_3: FfxResourceDescription__bindgen_ty_3 { depth: todo!() },
                mipCount: image.mip_level_count,
                flags: FfxResourceFlags_FFX_RESOURCE_FLAGS_NONE,
                usage: todo!(),
            },
            ptr::null_mut(),
            FfxResourceStates_FFX_RESOURCE_STATE_PIXEL_COMPUTE_READ,
        )
    }
}
