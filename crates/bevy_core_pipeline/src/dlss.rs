use crate::prepass::{DepthPrepass, VelocityPrepass};
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::{Bundle, Component};
use bevy_render::{
    camera::TemporalJitter, prelude::Msaa, renderer::RenderDevice, DlssAvailable, DlssProjectId,
    RenderApp,
};
use bevy_utils::tracing::info;
use dlss_wgpu::DlssSdk;

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
        let render_device = {
            app.get_sub_app_mut(RenderApp)
                .unwrap()
                .world
                .resource::<RenderDevice>()
                .clone()
        };

        let dlss_sdk = DlssSdk::new(project_id, render_device);
        if dlss_sdk.is_err() {
            app.world.remove_resource::<DlssAvailable>();
            info!("DLSS not available");
            return;
        }

        app.insert_resource(Msaa::Off);

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app.insert_non_send_resource(dlss_sdk.unwrap());
    }
}

#[derive(Bundle, Default)]
pub struct DlssBundle {
    pub settings: DlssSettings,
    pub jitter: TemporalJitter,
    pub depth_prepass: DepthPrepass,
    pub velocity_prepass: VelocityPrepass,
}

#[derive(Component, Clone, Default)]
pub struct DlssSettings {}
