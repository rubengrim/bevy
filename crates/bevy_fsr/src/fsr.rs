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

use bevy_render::renderer::{
    wgpu_hal_api::{Api, Vulkan},
    RenderDevice,
};
use fsr::{
    ffxGetDeviceVK, ffxGetInterfaceVK, ffxGetScratchMemorySizeVK, FfxErrorCodes_FFX_OK,
    FfxInterface, VkDeviceContext,
};
use std::mem::MaybeUninit;

// TODO
const MAX_CONTEXTS: usize = 1;

pub struct FsrInterface {
    interface: FfxInterface,
    scratch_memory: Box<[u8]>,
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
            }
        };

        unsafe { render_device.wgpu_device().as_hal::<Vulkan, _, _>(c) }
    }
}
