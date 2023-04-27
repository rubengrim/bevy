pub mod camera;
pub mod node;

use self::camera::SolariSettings;
use bevy_app::{App, Plugin};
use bevy_render::extract_component::ExtractComponentPlugin;

pub struct SolariRealtimePlugin;

impl Plugin for SolariRealtimePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<SolariSettings>::default());
    }
}
