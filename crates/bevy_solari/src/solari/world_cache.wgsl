/// Maximum amount of entries in the world cache (must be a power of 2)
const WORLD_CACHE_SIZE: u32 = 1048576u;
/// Maximum amount of frames a cell can live for without being queried
const WORLD_CACHE_CELL_LIFETIME: u32 = 10u;
/// Marker value for an empty cell
const WORLD_CACHE_EMPTY_CELL: u32 = 4294967295u;
/// Maximum amount of steps to linearly probe for on key collision before giving up
const WORLD_CACHE_MAX_SEARCH_STEPS: u32 = 10u;

@group(0) @binding(0)
var<storage, read_write> world_cache_checksums: array<atomic<u32>, WORLD_CACHE_SIZE>;

// Accessed as atomic in most passes, except for decrement_world_cache_cell_life
@group(0) @binding(1)
var<storage, read_write> world_cache_life: array<atomic<u32>, WORLD_CACHE_SIZE>;
@group(0) @binding(1)
var<storage, read_write> world_cache_life_non_atomic: array<u32, WORLD_CACHE_SIZE>;

@group(0) @binding(2)
var<storage, read_write> world_cache_irradiance: array<vec3<f32>, WORLD_CACHE_SIZE>;

struct WorldCacheExtraData {
    position: vec3<f32>,
}
@group(0) @binding(3)
var<storage, read_write> world_cache_extra_data: array<WorldCacheExtraData, WORLD_CACHE_SIZE>;

@group(0) @binding(4)
var<storage, read_write> world_cache_active_cells_new_irradiance: array<vec3<f32>, WORLD_CACHE_SIZE>;

// ------------------------------------------------------------------------------

fn pcg_hash(input: u32) -> u32 {
    let state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn iqint_hash(input: u32) -> u32 {
    let n = (input << 13u) ^ input;
    return n * (n * n * 15731u + 789221u) + 1376312589u;
}

fn wrap_key(key: u32) -> u32 {
    return key & (WORLD_CACHE_SIZE - 1u);
}

fn compute_key(world_position: vec3<f32>) -> u32 {
    let world_position_quantized = vec3<u32>(world_position / 8.0);
    var key = pcg_hash(world_position_quantized.x);
    key = pcg_hash(key + world_position_quantized.y);
    key = pcg_hash(key + world_position_quantized.z);
    return wrap_key(key);
}

fn compute_checksum(world_position: vec3<f32>) -> u32 {
    let world_position_quantized = vec3<u32>(world_position / 8.0);
    var key = iqint_hash(world_position_quantized.x);
    key = iqint_hash(key + world_position_quantized.y);
    key = iqint_hash(key + world_position_quantized.z);
    return key;
}

// ------------------------------------------------------------------------------

fn query_world_cache(world_position: vec3<f32>) -> vec3<f32> {
    var key = compute_key(world_position);
    let checksum = compute_checksum(world_position);

    for (var i = 0u; i < WORLD_CACHE_MAX_SEARCH_STEPS; i++) {
        let existing_checksum = atomicCompareExchangeWeak(&world_cache_checksums[key], checksum, WORLD_CACHE_EMPTY_CELL);
        if existing_checksum == checksum {
            // Key is already stored - get the corresponding irradiance and reset cell lifetime
            atomicStore(&world_cache_life[key], WORLD_CACHE_CELL_LIFETIME);
            return world_cache_irradiance[key];
        } else if existing_checksum == WORLD_CACHE_EMPTY_CELL {
            // Key is not stored - reset cell lifetime so that it starts getting updated next frame
            atomicStore(&world_cache_life[key], WORLD_CACHE_CELL_LIFETIME);
            world_cache_extra_data[key].position = world_position;
            return vec3(0.0);
        } else {
            // Collision - jump to next cell
            key = wrap_key(key + 1u); // TODO: Compare +1 vs hashing the key again
        }
    }

    return vec3(0.0);
}

fn trace_world_cache_cell_ray(active_cell_index: u32, cell_index: u32) {
    // TODO: Trace rays from cell position. If hit a point, add emittance, but also world cache irradiance
    // using query_world_cache()
}

fn blend_world_cache_cell_irradiance(active_cell_index: u32, cell_index: u32) {
    // TODO: Read sample from world_cache_active_cells_new_irradiance, blend with existing
    // irradiance in world_cache_irradiance
}

// ------------------------------------------------------------------------------

const WORLD_CACHE_SIZE: u32 = 1048576u;
const WORLD_CACHE_EMPTY_CELL: u32 = 4294967295u;

struct DispatchIndirect {
    x: u32,
    y: u32,
    z: u32,
}

var<storage, read_write> b1: array<u32, WORLD_CACHE_SIZE>;
var<storage, read_write> b2: array<u32, 1024u>;
var<workgroup> w1: array<u32, 1024u>;
var<workgroup> w2: array<u32, 1024u>;

var<storage, read_write> world_cache_active_cells: array<u32, WORLD_CACHE_SIZE>;
var<storage, read_write> world_cache_active_cell_count: u32;
var<storage, read_write> world_cache_active_cells_dispatch: DispatchIndirect;

@compute(1024, 1, 1)
fn decay_world_cache_cells(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var life = world_cache_life_non_atomic[global_id.x];
    if life > 0u {
        life -= 1u;
        world_cache_life_non_atomic[global_id.x] = life;

        if life == 0u {
            world_cache_life_non_atomic[global_id.x] = WORLD_CACHE_EMPTY_CELL;
            world_cache_irradiance[global_id.x] = vec3(0.0);
        }
    }
}

@compute(1024, 1, 1)
fn compact_world_cache_single_block(
    @builtin(global_invocation_id) cell_id: vec3<u32>,
    @builtin(local_invocation_index) t: u32,
) {
    if t == 0u { w1[0u] = 0u } else { w1[t] = world_cache_life_non_atomic[cell_id.x - 1u] != 0u }; workgroupBarrier();
    if t < 1u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 1u] } workgroupBarrier();
    if t < 2u { w1[t] = w2[t] } else { w1[t] = w2[t] + w2[t - 2u] } workgroupBarrier();
    if t < 4u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 4u] } workgroupBarrier();
    if t < 8u { w1[t] = w2[t] } else { w1[t] = w2[t] + w2[t - 8u] } workgroupBarrier();
    if t < 16u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 16u] } workgroupBarrier();
    if t < 32u { w1[t] = w2[t] } else { w1[t] = w2[t] + w2[t - 32u] } workgroupBarrier();
    if t < 64u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 64u] } workgroupBarrier();
    if t < 128u { w1[t] = w2[t] } else { w1[t] = w2[t] + w2[t - 128u] } workgroupBarrier();
    if t < 256u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 256u] } workgroupBarrier();
    if t < 512u { b1[t] = w2[t] } else { b1[t] = w2[t] + w2[t - 512u] }
}

@compute(1024, 1, 1)
fn compact_world_cache_blocks(@builtin(local_invocation_index) t: u32) {
    if t == 0u { w1[0u] = 0u } else { w1[t] = b1[t * 1024u - 1u] != 0u }; workgroupBarrier();
    if t < 1u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 1u] } workgroupBarrier();
    if t < 2u { w1[t] = w2[t] } else { w1[t] = w2[t] + w2[t - 2u] } workgroupBarrier();
    if t < 4u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 4u] } workgroupBarrier();
    if t < 8u { w1[t] = w2[t] } else { w1[t] = w2[t] + w2[t - 8u] } workgroupBarrier();
    if t < 16u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 16u] } workgroupBarrier();
    if t < 32u { w1[t] = w2[t] } else { w1[t] = w2[t] + w2[t - 32u] } workgroupBarrier();
    if t < 64u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 64u] } workgroupBarrier();
    if t < 128u { w1[t] = w2[t] } else { w1[t] = w2[t] + w2[t - 128u] } workgroupBarrier();
    if t < 256u { w2[t] = w1[t] } else { w2[t] = w1[t] + w1[t - 256u] } workgroupBarrier();
    if t < 512u { b2[t] = w2[t] } else { b2[t] = w2[t] + w2[t - 512u] }
}

var<workgroup> w_b2: u32;

@compute(1024, 1, 1)
fn compact_world_cache_write_active_cells(
    @builtin(global_invocation_id) cell_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_index) thread_index: u32,
) {
    if thread_index == 0u {
        w_b2 = b2[workgroup_id.x];
    }
    workgroupBarrier();

    let compacted_index = b1[cell_id.x] + w_b2;
    if world_cache_life_non_atomic[cell_id.x] != 0u {
        world_cache_active_cells[compacted_index] = cell_id.x;
    }

    if thread_index == 0u && workgroup_id.x == 0u {
        world_cache_active_cell_count = compacted_index + 1u;
        world_cache_active_cells_dispatch = DispatchIndirect((world_cache_active_cell_count + 1023u) / 1024u, 1u, 1u);
    }
}
