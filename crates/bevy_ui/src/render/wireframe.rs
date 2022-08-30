use bevy_app::Plugin;
use bevy_asset::{load_internal_asset, Handle, HandleUntyped};
use bevy_core_pipeline::core_3d::Opaque3d;
use bevy_ecs::{prelude::*, reflect::ReflectComponent};
use bevy_reflect::std_traits::ReflectDefault;
use bevy_reflect::{Reflect, TypeUuid};
use bevy_render::render_resource::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, ShaderStages, ShaderType, SpecializedRenderPipeline,
};
use bevy_render::renderer::RenderDevice;
use bevy_render::view::ViewUniform;
use bevy_render::Extract;
use bevy_render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    mesh::{Mesh, MeshVertexBufferLayout},
    render_asset::RenderAssets,
    render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
    render_resource::{
        PipelineCache, PolygonMode, RenderPipelineDescriptor, Shader, SpecializedMeshPipeline,
        SpecializedMeshPipelineError, SpecializedMeshPipelines,
    },
    view::{ExtractedView, Msaa, VisibleEntities},
    RenderApp, RenderStage,
};
use bevy_transform::prelude::Transform;
use bevy_utils::tracing::error;

use crate::{DrawUi, Node, SetUiTextureBindGroup, SetUiViewBindGroup, UiPipeline, UiPipelineKey};

pub const WIREFRAME_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 192598014480025767);

#[derive(Debug, Default)]
pub struct UiWireframePlugin;
impl Plugin for UiWireframePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        load_internal_asset!(
            app,
            WIREFRAME_SHADER_HANDLE,
            "wireframe.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<UiWireframeConfig>()
            .init_resource::<UiWireframeConfig>()
            .add_plugin(ExtractResourcePlugin::<UiWireframeConfig>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Opaque3d, DrawWireframes>()
                .init_resource::<UiWireframePipeline>()
                .init_resource::<SpecializedMeshPipelines<UiWireframePipeline>>()
                .add_system_to_stage(RenderStage::Extract, extract_wireframes)
                .add_system_to_stage(RenderStage::Queue, queue_wireframes);
        }
    }
}

fn extract_wireframes(mut commands: Commands, query: Extract<Query<Entity, With<Wireframe>>>) {
    for entity in query.iter() {
        commands.get_or_spawn(entity).insert(Wireframe);
    }
}

/// Controls whether an entity should rendered in wireframe-mode if the [`WireframePlugin`] is enabled
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component, Default)]
pub struct Wireframe;

#[derive(Resource, Debug, Clone, Default, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct UiWireframeConfig {
    /// Whether to show wireframes for all meshes. If `false`, only meshes with a [Wireframe] component will be rendered.
    pub global: bool,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct UiWireframePipelineKey {}

#[derive(Resource)]
pub struct UiWireframePipeline {
    pub ui_pipeline: UiPipeline,
    shader: Handle<Shader>,
}

impl FromWorld for UiWireframePipeline {
    fn from_world(world: &mut World) -> Self {
        UiWireframePipeline {
            ui_pipeline: world.resource::<UiPipeline>().clone(),
            shader: WIREFRAME_SHADER_HANDLE.typed(),
        }
    }
}

impl SpecializedMeshPipeline for UiWireframePipeline {
    type Key = UiPipelineKey;
    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.ui_pipeline.specialize(key);
        descriptor.vertex.shader = self.shader.clone_weak();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone_weak();
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        descriptor.depth_stencil.as_mut().unwrap().bias.slope_scale = 1.0;
        Ok(descriptor)
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_wireframes(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    wireframe_config: Res<UiWireframeConfig>,
    wireframe_pipeline: Res<UiWireframePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<UiWireframePipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut material_meshes: ParamSet<(
        Query<(Entity, &Node, &Transform)>,
        Query<(Entity, &Node, &Transform), With<Wireframe>>,
    )>,
    mut views: Query<(&ExtractedView, &VisibleEntities, &mut RenderPhase<Opaque3d>)>,
) {
    let draw_custom = opaque_3d_draw_functions
        .read()
        .get_id::<DrawWireframes>()
        .unwrap();
    for (view, visible_entities, mut opaque_phase) in &mut views {
        let rangefinder = view.rangefinder3d();

        let add_render_phase = |(entity, node, transform): (Entity, &Node, &Transform)| {
            let pipeline_id =
                pipelines.specialize(&mut pipeline_cache, &wireframe_pipeline, key, &mesh.layout);
            opaque_phase.add(Opaque3d {
                entity,
                pipeline: pipeline_id,
                draw_function: draw_custom,
                distance: 1.0,
            });
        };

        if wireframe_config.global {
            let query = material_meshes.p0();
            visible_entities
                .entities
                .iter()
                .filter_map(|visible_entity| query.get(*visible_entity).ok())
                .for_each(add_render_phase);
        } else {
            let query = material_meshes.p1();
            visible_entities
                .entities
                .iter()
                .filter_map(|visible_entity| query.get(*visible_entity).ok())
                .for_each(add_render_phase);
        }
    }
}

type DrawWireframes = (
    SetItemPipeline,
    SetUiViewBindGroup<0>,
    SetUiTextureBindGroup<1>,
    DrawUi,
);
