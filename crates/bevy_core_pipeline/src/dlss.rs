use crate::prepass::{DepthPrepass, VelocityPrepass};
use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::{Bundle, Component},
    system::Resource,
};
use bevy_render::{camera::TemporalJitter, prelude::Msaa, renderer::RenderDevice, RenderApp};
use dlss_wgpu::DlssSdk;
use uuid::Uuid;

mod draw_3d_graph {
    pub mod node {
        /// Label for the DLSS render node.
        pub const DLSS: &str = "dlss";
    }
}

pub struct DlssPlugin {
    pub project_id: Uuid,
}

impl Plugin for DlssPlugin {
    fn build(&self, app: &mut App) {
        if app.get_sub_app_mut(RenderApp).is_err() {
            return;
        }

        let render_device = {
            app.get_sub_app_mut(RenderApp)
                .unwrap()
                .world
                .resource::<RenderDevice>()
                .clone()
        };

        let dlss_sdk = DlssSdk::new(self.project_id, render_device);

        if dlss_sdk.is_ok() {
            app.insert_resource(DlssAvailable)
                .insert_resource(Msaa::Off);
        } else {
            return;
        }

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app.insert_non_send_resource(DlssSdkResource(dlss_sdk.unwrap()));

        // let fsr2_node = Fsr2Node::new(&mut render_app.world);
        // let mut graph = render_app.world.resource_mut::<RenderGraph>();
        // let draw_3d_graph = graph
        //     .get_sub_graph_mut(crate::core_3d::graph::NAME)
        //     .unwrap();
        // draw_3d_graph.add_node(draw_3d_graph::node::FSR2, fsr2_node);
        // draw_3d_graph.add_slot_edge(
        //     draw_3d_graph.input_node().id,
        //     crate::core_3d::graph::input::VIEW_ENTITY,
        //     draw_3d_graph::node::FSR2,
        //     Fsr2Node::IN_VIEW,
        // );
        // // MAIN_PASS -> FSR2 -> BLOOM / TONEMAPPING
        // draw_3d_graph.add_node_edge(
        //     crate::core_3d::graph::node::MAIN_PASS,
        //     draw_3d_graph::node::FSR2,
        // );
        // draw_3d_graph.add_node_edge(
        //     draw_3d_graph::node::FSR2,
        //     crate::core_3d::graph::node::BLOOM,
        // );
        // draw_3d_graph.add_node_edge(
        //     draw_3d_graph::node::FSR2,
        //     crate::core_3d::graph::node::TONEMAPPING,
        // );

        // render_app
        //     .init_resource::<Fsr2ContextCache>()
        //     .add_system_to_schedule(ExtractSchedule, extract_fsr2_settings)
        //     .add_system(
        //         prepare_fsr2
        //             .in_set(RenderSet::Prepare)
        //             .before(prepare_core_3d_textures),
        //     );
    }
}

#[derive(Resource)]
pub struct DlssAvailable;

#[derive(Bundle, Default)]
pub struct DlssBundle {
    pub settings: DlssSettings,
    pub jitter: TemporalJitter,
    pub depth_prepass: DepthPrepass,
    pub velocity_prepass: VelocityPrepass,
}

#[derive(Component, Clone, Default)]
pub struct DlssSettings {}

struct DlssSdkResource(DlssSdk<RenderDevice>);
