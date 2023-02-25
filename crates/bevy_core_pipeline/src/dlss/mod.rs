use bevy_app::{App, Plugin};

mod draw_3d_graph {
    pub mod node {
        /// Label for the DLSS render node.
        pub const DLSS: &str = "dlss";
    }
}

pub struct DLSSPlugin;

impl Plugin for DLSSPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "dlss")]
        {
            use bevy_render::{view::Msaa, RenderApp};

            if app.get_sub_app_mut(RenderApp).is_err() {
                return;
            }

            app.insert_resource(Msaa::Off);

            let render_app = app.get_sub_app_mut(RenderApp).unwrap();

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
}
