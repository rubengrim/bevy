pub mod camera;
pub mod node;
mod pipeline;
mod resources;

use crate::path_tracer::pipeline::TraceRaysFromBuffer;

use self::{
    camera::{reset_accumulation_on_camera_movement, SolariPathTracer},
    pipeline::{
        prepare_pipelines, prepare_pipelines2, prepare_pipelines3, prepare_pipelines4,
        prepare_pipelines5, prepare_pipelines6, prepare_pipelines7, prepare_pipelines8,
        SolariPathtracerPipeline,
    },
    resources::prepare_accumulation_textures,
};
use bevy_app::{App, Plugin, PostUpdate};
use bevy_asset::{load_internal_asset, HandleUntyped};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_reflect::TypeUuid;
use bevy_render::{
    extract_component::ExtractComponentPlugin,
    render_resource::{Shader, SpecializedComputePipelines},
    Render, RenderApp, RenderSet,
};
use bevy_transform::TransformSystem;

pub struct SolariPathTracerPlugin;

const SOLARI_PATH_TRACER_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3717171717171755);
const SOLARI_TRACE_RAYS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5517471717171755);
const SOLARI_SORT_RAYS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4715557147171743);

impl Plugin for SolariPathTracerPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SOLARI_PATH_TRACER_SHADER,
            "path_tracer.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_TRACE_RAYS_SHADER,
            "trace_rays.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_SORT_RAYS_SHADER,
            "sort_rays.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(ExtractComponentPlugin::<SolariPathTracer>::default())
            .add_systems(
                PostUpdate,
                reset_accumulation_on_camera_movement.after(TransformSystem::TransformPropagate),
            );

        app.sub_app_mut(RenderApp)
            .init_resource::<SolariPathtracerPipeline>()
            .init_resource::<SpecializedComputePipelines<SolariPathtracerPipeline>>()
            .init_resource::<TraceRaysFromBuffer>()
            .init_resource::<SpecializedComputePipelines<TraceRaysFromBuffer>>()
            .add_systems(
                Render,
                (
                    prepare_accumulation_textures,
                    prepare_pipelines,
                    prepare_pipelines2,
                    prepare_pipelines3,
                    prepare_pipelines4,
                    prepare_pipelines5,
                    prepare_pipelines6,
                    prepare_pipelines7,
                    prepare_pipelines8,
                )
                    .in_set(RenderSet::Prepare),
            );
    }
}
