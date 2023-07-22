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
use bevy_render::renderer::{
    wgpu_hal_api::{Api, Vulkan},
    RenderDevice,
};
use fsr::{
    ffxFsr2ContextCreate, ffxFsr2ContextDestroy, ffxFsr2GetJitterOffset,
    ffxFsr2GetJitterPhaseCount, ffxGetDeviceVK, ffxGetInterfaceVK, ffxGetScratchMemorySizeVK,
    FfxDimensions2D, FfxErrorCodes_FFX_OK, FfxFsr2Context, FfxFsr2ContextDescription,
    FfxFsr2InitializationFlagBits, FfxInterface, VkDeviceContext,
};
use std::{mem::MaybeUninit, ops::Sub};

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

    pub fn dispatch(&mut self) {
        todo!()
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
