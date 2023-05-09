#define_import_path bevy_solari::world_cache::utils

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

#ifndef WORLD_CACHE_NON_ATOMIC_LIFE_BUFFER
fn query_world_cache(world_position: vec3<f32>) -> vec3<f32> {
    var key = compute_key(world_position);
    let checksum = compute_checksum(world_position);

    for (var i = 0u; i < WORLD_CACHE_MAX_SEARCH_STEPS; i++) {
        let existing_checksum = atomicCompareExchangeWeak(&world_cache_checksums[key], WORLD_CACHE_EMPTY_CELL, checksum).old_value;
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
#endif
