struct WorldCacheHeader {
    checksum: atomic<u32>,
    last_change_index: atomic<u32>,
}

struct WorldCacheChangelist {
    sampled_radiance: vec3<f32>,
    previous_change_index: atomic<u32>,
}

struct WorldCacheData {
    life: atomic<u32>,
    position: vec3<f32>,
    irradiance: vec3<f32>,
}

/// Maximum amount of entries in the world cache (must be a power of 2)
const WORLD_CACHE_SIZE: u32 = 4194304u;
/// Maximum amount of frames a cell can live for without being queried
const WORLD_CACHE_CELL_LIFETIME: u32 = 10u;
/// Marker value for an empty cell or previous changelist
const WORLD_CACHE_NULL: u32 = 4294967295u;
/// Maximum amount of steps to linearly probe for on key collision before giving up
const WORLD_CACHE_MAX_SEARCH_STEPS: u32 = 10u;

@group(0) @binding(0)
var<storage, read_write> world_cache_headers: array<WorldCacheHeader, WORLD_CACHE_SIZE>;
@group(0) @binding(1)
var<storage, read_write> world_cache_data: array<WorldCacheData, WORLD_CACHE_SIZE>;
@group(0) @binding(2)
var<storage, read_write> world_cache_changelist: array<WorldCacheChangelist, WORLD_CACHE_SIZE>;

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
        let existing_checksum = atomicCompareExchangeWeak(&world_cache_headers[key].checksum, checksum, WORLD_CACHE_NULL);
        if existing_checksum == checksum {
            atomicStore(&world_cache_data[key].life, WORLD_CACHE_CELL_LIFETIME);
            return world_cache_data[key].irradiance;
        } else if existing_checksum == WORLD_CACHE_NULL {
            atomicStore(&world_cache_data[key].life, WORLD_CACHE_CELL_LIFETIME);
            world_cache_data[key].position = world_position;
            return world_cache_data[key].irradiance;
        } else {
            key = wrap_key(pcg_hash(key));
        }
    }

    return vec3(0.0);
}
