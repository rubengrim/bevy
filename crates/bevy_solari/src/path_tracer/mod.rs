mod bind_group;
pub mod camera;
pub mod node;
mod pipeline;

use self::{
    camera::SolariPathTracer,
    pipeline::{prepare_pipelines, SolariPathtracerPipeline},
};
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_reflect::TypeUuid;
use bevy_render::{
    extract_component::ExtractComponentPlugin,
    render_resource::{Shader, SpecializedComputePipelines},
    Render, RenderApp, RenderSet,
};

pub struct SolariPathTracerPlugin;

const SOLARI_PATH_TRACER_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3717171717171755);

impl Plugin for SolariPathTracerPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SOLARI_PATH_TRACER_SHADER,
            "path_tracer.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(ExtractComponentPlugin::<SolariPathTracer>::default());

        app.sub_app_mut(RenderApp)
            .init_resource::<SolariPathtracerPipeline>()
            .init_resource::<SpecializedComputePipelines<SolariPathtracerPipeline>>()
            .add_systems(Render, prepare_pipelines.in_set(RenderSet::Prepare));
    }
}
