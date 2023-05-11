pub mod node;
mod pipelines;
pub mod resources;

use self::{pipelines::SolariWorldCachePipelineIds, resources::SolariWorldCacheResources};
use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, HandleUntyped};

use bevy_reflect::TypeUuid;
use bevy_render::{render_resource::Shader, RenderApp};

const WORLD_CACHE_SIZE: u64 = 1048576;

pub struct SolariWorldCachePlugin;

const SOLARI_WORLD_CACHE_BINDINGS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1717171717171756);
const SOLARI_WORLD_CACHE_UTILS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2717171717171756);
const SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3717171717171756);
const SOLARI_WORLD_CACHE_UPDATE_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4717171717171756);

impl Plugin for SolariWorldCachePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SOLARI_WORLD_CACHE_BINDINGS_SHADER,
            "world_cache_bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_WORLD_CACHE_UTILS_SHADER,
            "world_cache_utils.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER,
            "compact_active_cells.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SOLARI_WORLD_CACHE_UPDATE_SHADER,
            "update_world_cache.wgsl",
            Shader::from_wgsl
        );

        app.sub_app_mut(RenderApp)
            .init_resource::<SolariWorldCacheResources>()
            .init_resource::<SolariWorldCachePipelineIds>();
    }
}
