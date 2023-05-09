use super::{resources::SolariWorldCacheResources, SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER};
use bevy_ecs::{
    system::Resource,
    world::{FromWorld, World},
};
use bevy_render::render_resource::{
    CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderDefVal,
};

#[derive(Resource)]
pub struct SolariWorldCachePipelineIds {
    pub decay_world_cache_cells: CachedComputePipelineId,
    pub compact_world_cache_single_block: CachedComputePipelineId,
    pub compact_world_cache_blocks: CachedComputePipelineId,
    pub compact_world_cache_write_active_cells: CachedComputePipelineId,
}

impl FromWorld for SolariWorldCachePipelineIds {
    fn from_world(world: &mut World) -> Self {
        let pipeline_cache = world.resource::<PipelineCache>();
        let world_cache_resources = world.resource::<SolariWorldCacheResources>();

        let decay_world_cache_cells = ComputePipelineDescriptor {
            label: Some("solari_decay_world_cache_cells_pipeline".into()),
            layout: vec![world_cache_resources.bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader: SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER.typed(),
            shader_defs: vec![
                "WORLD_CACHE_NON_ATOMIC_LIFE_BUFFER".into(),
                ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 0),
            ],
            entry_point: "decay_world_cache_cells".into(),
        };

        let compact_world_cache_single_block = ComputePipelineDescriptor {
            label: Some("solari_compact_world_cache_single_block_pipeline".into()),
            layout: vec![world_cache_resources.bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader: SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER.typed(),
            shader_defs: vec![
                "WORLD_CACHE_NON_ATOMIC_LIFE_BUFFER".into(),
                ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 0),
            ],
            entry_point: "compact_world_cache_single_block".into(),
        };

        let compact_world_cache_blocks = ComputePipelineDescriptor {
            label: Some("solari_compact_world_cache_blocks_pipeline".into()),
            layout: vec![world_cache_resources.bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader: SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER.typed(),
            shader_defs: vec![
                "WORLD_CACHE_NON_ATOMIC_LIFE_BUFFER".into(),
                ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 0),
            ],
            entry_point: "compact_world_cache_blocks".into(),
        };

        let compact_world_cache_write_active_cells = ComputePipelineDescriptor {
            label: Some("solari_compact_world_cache_write_active_cells_pipeline".into()),
            layout: vec![world_cache_resources.bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader: SOLARI_WORLD_CACHE_COMPACT_ACTIVE_CELLS_SHADER.typed(),
            shader_defs: vec![
                "WORLD_CACHE_NON_ATOMIC_LIFE_BUFFER".into(),
                ShaderDefVal::UInt("WORLD_CACHE_BIND_GROUP".into(), 0),
            ],
            entry_point: "compact_world_cache_write_active_cells".into(),
        };

        Self {
            decay_world_cache_cells: pipeline_cache.queue_compute_pipeline(decay_world_cache_cells),
            compact_world_cache_single_block: pipeline_cache
                .queue_compute_pipeline(compact_world_cache_single_block),
            compact_world_cache_blocks: pipeline_cache
                .queue_compute_pipeline(compact_world_cache_blocks),
            compact_world_cache_write_active_cells: pipeline_cache
                .queue_compute_pipeline(compact_world_cache_write_active_cells),
        }
    }
}
