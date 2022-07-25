mod diagnostic;
mod entity_count_diagnostics_plugin;
mod frame_time_diagnostics_plugin;
mod log_diagnostics_plugin;
use bevy_log::info;
pub use diagnostic::*;
pub use entity_count_diagnostics_plugin::EntityCountDiagnosticsPlugin;
pub use frame_time_diagnostics_plugin::FrameTimeDiagnosticsPlugin;
pub use log_diagnostics_plugin::LogDiagnosticsPlugin;

use bevy_app::prelude::*;

/// Adds core diagnostics resources to an App.
#[derive(Default)]
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Diagnostics>()
            .add_startup_system(log_system_info);
    }
}

/// The width which diagnostic names will be printed as
/// Plugin names should not be longer than this value
pub const MAX_DIAGNOSTIC_NAME_WIDTH: usize = 32;

#[derive(Debug)]
// This is required because the Debug trait doesn't detect it's used when it's only used in a print :(
#[allow(dead_code)]
struct SystemInfo {
    os: String,
    kernel: String,
    cpu: String,
    core_count: String,
    memory: String,
}

fn log_system_info() {
    use sysinfo::{CpuExt, SystemExt};

    let mut sys = sysinfo::System::new();
    sys.refresh_cpu();
    sys.refresh_memory();

    let info = SystemInfo {
        os: sys
            .long_os_version()
            .unwrap_or_else(|| String::from("not available")),
        kernel: sys
            .kernel_version()
            .unwrap_or_else(|| String::from("not available")),
        cpu: sys.global_cpu_info().brand().trim().to_string(),
        core_count: sys
            .physical_core_count()
            .map(|x| x.to_string())
            .unwrap_or_else(|| String::from("not available")),
        memory: format!("{} KB", sys.total_memory()),
    };

    info!("{:?}", info);
}
