use bevy_ecs::{
    system::Resource,
    world::{FromWorld, World},
};
use bevy_render::{render_resource::*, renderer::RenderDevice};
use std::num::NonZeroU64;

pub const WORLD_CACHE_SIZE: u64 = 1048576;

#[derive(Resource)]
pub struct SolariWorldCacheResources {
    pub bind_group_layout: BindGroupLayout,
    pub bind_group_layout_no_dispatch: BindGroupLayout,
    pub bind_group: BindGroup,
    pub bind_group_no_dispatch: BindGroup,
    pub active_cells_dispatch_buffer: Buffer,
}

impl FromWorld for SolariWorldCacheResources {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout_entries = &[
            // Checksums
            bgl_entry(0, 4),
            // Life
            bgl_entry(1, 4),
            // Irradiance
            bgl_entry(2, 16),
            // Cell data
            bgl_entry(3, 32),
            // Active cells new irradiance
            bgl_entry(4, 16),
            // B1
            bgl_entry(5, 4),
            // B2
            bgl_entry(6, 4),
            // Active cell indices
            bgl_entry(7, 4),
            // Active cells count
            bgl_entry(8, 4),
            // Active cells dispatch
            bgl_entry(9, 12),
        ];

        let create_buffer = |label, size| {
            render_device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        };

        let checksums = create_buffer("bevy_solari_world_cache_checksums", 4 * WORLD_CACHE_SIZE);
        let life = create_buffer("bevy_solari_world_cache_life", 4 * WORLD_CACHE_SIZE);
        let irradiance = create_buffer("bevy_solari_world_cache_irradiance", 16 * WORLD_CACHE_SIZE);
        let cell_data = create_buffer("bevy_solari_world_cache_cell_data", 32 * WORLD_CACHE_SIZE);
        let active_cells_new_irradiance = create_buffer(
            "bevy_solari_world_cache_active_cells_new_irradiance",
            16 * WORLD_CACHE_SIZE,
        );
        let b1 = create_buffer("bevy_solari_world_cache_b1", 4 * WORLD_CACHE_SIZE);
        let b2 = create_buffer("bevy_solari_world_cache_b2", 4 * 1024);
        let active_cell_indices = create_buffer(
            "bevy_solari_world_cache_active_cell_indices",
            4 * WORLD_CACHE_SIZE,
        );
        let active_cells_count = create_buffer("bevy_solari_world_cache_active_cells_count", 4);
        let active_cells_dispatch_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("bevy_solari_world_cache_active_cells_dispatch_buffer"),
            size: 12,
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        let bind_group_entries = &[
            bg_entry(0, &checksums),
            bg_entry(1, &life),
            bg_entry(2, &irradiance),
            bg_entry(3, &cell_data),
            bg_entry(4, &active_cells_new_irradiance),
            bg_entry(5, &b1),
            bg_entry(6, &b2),
            bg_entry(7, &active_cell_indices),
            bg_entry(8, &active_cells_count),
            bg_entry(9, &active_cells_dispatch_buffer),
        ];

        let bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("solari_world_cache_bind_group_layout"),
                entries: bind_group_layout_entries,
            });

        let bind_group_layout_no_dispatch =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("solari_world_cache_bind_group_layout_no_dispatch"),
                entries: &bind_group_layout_entries[0..bind_group_entries.len() - 1],
            });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("solari_world_cache_bind_group"),
            layout: &bind_group_layout,
            entries: bind_group_entries,
        });

        let bind_group_no_dispatch = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("solari_world_cache_bind_group_no_dispatch"),
            layout: &bind_group_layout_no_dispatch,
            entries: &bind_group_entries[0..bind_group_entries.len() - 1],
        });

        Self {
            bind_group_layout,
            bind_group_layout_no_dispatch,
            bind_group,
            bind_group_no_dispatch,
            active_cells_dispatch_buffer,
        }
    }
}

fn bgl_entry(binding: u32, min_binding_size: u64) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(min_binding_size) }),
        },
        count: None,
    }
}

fn bg_entry(binding: u32, buffer: &Buffer) -> BindGroupEntry {
    BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}
