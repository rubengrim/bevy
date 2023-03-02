pub use bevy_render::{DlssAvailable, DlssProjectId};
pub use dlss_wgpu::DlssPreset;

use crate::{
    prelude::Camera3d,
    prepass::{DepthPrepass, MotionVectorPrepass},
};
use bevy_app::{App, IntoSystemAppConfig, Plugin};
use bevy_ecs::{
    prelude::{Bundle, Component, Entity, NonSendMut, Query},
    query::With,
    schedule::IntoSystemConfig,
    system::{Commands, ResMut},
};
use bevy_render::{
    camera::TemporalJitter,
    prelude::{Camera, Msaa, Projection},
    renderer::RenderDevice,
    view::ExtractedView,
    ExtractSchedule, MainWorld, RenderApp, RenderSet,
};
use bevy_utils::tracing::info;
use dlss_wgpu::{DlssContext, DlssSdk};

mod draw_3d_graph {
    pub mod node {
        /// Label for the DLSS render node.
        pub const DLSS: &str = "dlss";
    }
}

pub struct DlssPlugin;

impl Plugin for DlssPlugin {
    fn build(&self, app: &mut App) {
        if app.get_sub_app_mut(RenderApp).is_err() {
            return;
        }
        if app.world.get_resource::<DlssAvailable>().is_none() {
            info!("DLSS not available");
            return;
        }

        let project_id = app.world.resource::<DlssProjectId>().0;
        let render_device = app
            .get_sub_app_mut(RenderApp)
            .unwrap()
            .world
            .resource::<RenderDevice>()
            .clone();

        let dlss_sdk = DlssSdk::new(project_id, render_device);
        if dlss_sdk.is_err() {
            app.world.remove_resource::<DlssAvailable>();
            info!("DLSS not available");
            return;
        }

        app.insert_resource(Msaa::Off);

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .insert_non_send_resource(dlss_sdk.unwrap())
            .add_system(extract_dlss_settings.in_schedule(ExtractSchedule))
            .add_system(prepare_dlss.in_set(RenderSet::Prepare));
    }
}

#[derive(Bundle, Default)]
pub struct DlssBundle {
    pub settings: DlssSettings,
    pub jitter: TemporalJitter,
    pub depth_prepass: DepthPrepass,
    pub motion_vector_prepass: MotionVectorPrepass,
}

#[derive(Component, Clone, Default)]
pub struct DlssSettings {
    pub preset: DlssPreset,
    pub reset: bool,
}

fn extract_dlss_settings(mut commands: Commands, mut main_world: ResMut<MainWorld>) {
    let mut query = main_world
        .query_filtered::<(Entity, &Camera, &Projection, &mut DlssSettings), (
            With<Camera3d>,
            With<TemporalJitter>,
            With<DepthPrepass>,
            With<MotionVectorPrepass>,
        )>();

    for (entity, camera, camera_projection, mut dlss_settings) in query.iter_mut(&mut main_world) {
        let has_perspective_projection = matches!(camera_projection, Projection::Perspective(_));
        if camera.is_active && has_perspective_projection {
            commands
                .get_or_spawn(entity)
                .insert((dlss_settings.clone(), camera_projection.clone()));

            dlss_settings.reset = false;
        }
    }
}

fn prepare_dlss(
    dlss_sdk: NonSendMut<DlssSdk<RenderDevice>>,
    mut query: Query<(Entity, &ExtractedView, &DlssSettings, &mut TemporalJitter)>,
) {
    // TODO: Create DLSS context, set TemporalJitter, and add ViewportOverride
}
