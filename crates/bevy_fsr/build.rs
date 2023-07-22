use bindgen::Builder;
use std::env;
use std::path::PathBuf;

fn main() {
    // Get paths
    let vulkan_sdk = env::var("VULKAN_SDK").expect("VULKAN_SDK environment variable not set");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Link to needed libraries
    println!("cargo:rustc-link-search=native=lib");
    #[cfg(not(target_os = "windows"))]
    {
        todo!();
    }
    #[cfg(target_os = "windows")]
    {
        #[cfg(not(debug_assertions))]
        println!("cargo:rustc-link-lib=static=ffx_fsr2_x64");
        #[cfg(debug_assertions)]
        println!("cargo:rustc-link-lib=static=ffx_fsr2_x64d");

        #[cfg(not(debug_assertions))]
        println!("cargo:rustc-link-lib=static=ffx_backend_vk_x64");
        #[cfg(debug_assertions)]
        println!("cargo:rustc-link-lib=static=ffx_backend_vk_x64d");

        println!("cargo:rustc-link-search=native={vulkan_sdk}\\Lib");
        println!("cargo:rustc-link-lib=dylib=vulkan-1");
    }

    #[cfg(not(target_os = "windows"))]
    let vulkan_sdk_include = "include";
    #[cfg(target_os = "windows")]
    let vulkan_sdk_include = "Include";

    // Generate rust bindings
    Builder::default()
        .header("include/bindgen.h")
        .clang_arg(format!("-I{vulkan_sdk}/{vulkan_sdk_include}"))
        .clang_args(["-x", "c++"])
        .clang_arg(format!("-Iinclude"))
        .bitfield_enum("FfxFsr2InitializationFlagBits")
        .blocklist_type("VkDevice")
        .blocklist_type("VkPhysicalDevice")
        .blocklist_type("PFN_vkGetDeviceProcAddr")
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .unwrap();
}
