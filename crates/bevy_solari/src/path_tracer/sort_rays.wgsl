#import bevy_solari::scene_bindings
#import bevy_render::view
#import bevy_solari::utils

const number_of_blocks: u32 = 4096u;
const block_size: u32 = 64u;
const number_of_elements: u32 = 262144u;
const number_of_blocks_p1: u32= 4097u;

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var accumulation_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(2)
var output_texture: texture_storage_2d<rgba16float, read_write>;

@group(1) @binding(3) var<storage, read_write> rays: array<RayDesc2>;
@group(1) @binding(4) var<storage, read_write> keys32: array<u32,number_of_elements>;
@group(1) @binding(5) var<storage, read_write> block_start_for_radix: array<u32,number_of_elements>;
@group(1) @binding(6) var<storage, read_write> is_ordered: array<atomic<u32>,1>;
@group(1) @binding(7) var<storage, read_write> starting_bit: array<u32,1>;
@group(1) @binding(8) var<storage, read_write> scan_results: array<array<u32,4>,number_of_blocks>;
@group(1) @binding(9) var<storage,read_write> prefix_sum_array: array<array<u32,4>,number_of_elements>;
@group(1) @binding(10) var<storage,read_write> rays_output: array<RayDesc2,number_of_elements>;
@group(1) @binding(11) var<storage,read_write> radix_block_info: array<array<u32,4>,number_of_elements>;
@group(1) @binding(12) var<storage,read_write> block_start_for_radix_output: array<u32,number_of_elements>;
@group(1) @binding(13) var<storage,read_write> keys32_out: array<u32,number_of_elements>;
@group(1) @binding(14) var<storage,read_write> id_of_id: array<u32,number_of_elements>;
@group(1) @binding(15) var<storage,read_write> block_sums: array<array<u32,4>,number_of_blocks>;

fn generate_id(wgid: vec3<u32>, lid: u32) -> u32 {
	return lid;
}

fn scalar_wgid(wgid: vec3<u32>) -> u32 {
	return wgid.x * 8u + wgid.y;
}

fn origin_key(origin: vec4<f32>,direction: vec4<f32>) -> u32 {
	let x = bitcast<u32>(origin.x);
	let y = bitcast<u32>(origin.y);
	let z = bitcast<u32>(origin.z);
	let tripleted: u32 = (x & 0x80000000u) | ((y >> 1u) & 0x40000000u) | ((z >> 2u) & 0x20000000u) |
						((x >> 2u) & 0x10000000u) | ((y >> 3u) & 0x08000000u) | ((z >> 4u) & 0x04000000u) |
						((x >> 4u) & 0x02000000u) | ((y >> 5u) & 0x01000000u) | ((z >> 6u) & 0x00800000u) |
						((x >> 6u) & 0x00400000u) | ((y >> 7u) & 0x00200000u) | ((z >> 8u) & 0x00100000u) |
						((x >> 8u) & 0x00080000u) | ((y >> 9u) & 0x00040000u) | ((z >> 10u) & 0x00020000u) |
						((x >> 10u) & 0x00010000u) | ((y >> 11u) & 0x00008000u) | ((z >> 12u) & 0x00004000u) |
						((x >> 12u) & 0x00002000u) | ((y >> 13u) & 0x00001000u) | ((z >> 14u) & 0x00000800u) |
						((x >> 14u) & 0x00000400u) | ((y >> 15u) & 0x00000200u) | ((z >> 16u) & 0x00000100u) |
						((x >> 16u) & 0x00000080u) | ((y >> 17u) & 0x00000040u) | ((z >> 18u) & 0x00000020u) |
						((x >> 18u) & 0x00000010u) | ((y >> 19u) & 0x00000008u) | ((z >> 20u) & 0x00000004u) |
						((x >> 20u) & 0x00000002u) | ((y >> 21u) & 0x00000001u);
	return tripleted;
}

fn origin_direction_key(origin: vec4<f32>,direction: vec4<f32>) -> u32 {
	let x = bitcast<u32>(origin.x);
	let y = bitcast<u32>(origin.y);
	let z = bitcast<u32>(origin.z);
	let dx = bitcast<u32>(direction.x);
	let dy = bitcast<u32>(direction.y);
	let dz = bitcast<u32>(direction.z);
	let tripleted: u32 = (x & 0x80000000u) | ((y >> 1u) & 0x40000000u) | ((z >> 2u) & 0x20000000u) |
						((x >> 2u) & 0x10000000u) | ((y >> 3u) & 0x08000000u) | ((z >> 4u) & 0x04000000u) |
						((x >> 4u) & 0x02000000u) | ((y >> 5u) & 0x01000000u) | ((z >> 6u) & 0x00800000u) |
						((x >> 6u) & 0x00400000u) | ((y >> 7u) & 0x00200000u) | ((z >> 8u) & 0x00100000u) |
						((x >> 8u) & 0x00080000u) | ((y >> 9u) & 0x00040000u) | ((z >> 10u) & 0x00020000u) |
						((x >> 10u) & 0x00010000u) | ((y >> 11u) & 0x00008000u) | ((z >> 12u) & 0x00004000u) |
						((x >> 12u) & 0x00002000u) | ((y >> 13u) & 0x00001000u) | ((z >> 14u) & 0x00000800u) |
						((x >> 14u) & 0x00000400u) |
						((dx >> 21u) & 0x00000200u) | ((dy >> 22u) & 0x00000100u) | ((dz >> 23u) & 0x00000080u) |
						((dx >> 23u) & 0x00000040u) | ((dy >> 24u) & 0x00000020u) | ((dz >> 25u) & 0x00000010u) |
						((dx >> 25u) & 0x00000008u) | ((dy >> 26u) & 0x00000004u) | ((dz >> 27u) & 0x00000002u) |
						((dx >> 27u) & 0x00000001u);
	return tripleted;
}

fn direction_origin_key(origin: vec4<f32>,direction: vec4<f32>) -> u32 {
	let x = bitcast<u32>(origin.x);
	let y = bitcast<u32>(origin.y);
	let z = bitcast<u32>(origin.z);
	let dx = bitcast<u32>(direction.x);
	let dy = bitcast<u32>(direction.y);
	let dz = bitcast<u32>(direction.z);
	let tripleted: u32 = ((dx) & 0x80000000u) | ((dy >> 1u) & 0x40000000u) | ((dz >> 2u) & 0x20000000u) |
						((dx >> 2u) & 0x10000000u) | ((dy >> 3u) & 0x08000000u) | ((dz >> 4u) & 0x04000000u) |
						((dx >> 4u) & 0x02000000u) | ((dy >> 5u) & 0x01000000u) |
						((x >> 8u) & 0x00800000u) | ((y >> 9u) & 0x00400000u) | ((z >> 10u) & 0x00200000u) |
						((x >> 10u) & 0x00100000u) | ((y >> 11u) & 0x00080000u) | ((z >> 12u) & 0x00040000u) |
						((x >> 12u) & 0x00020000u) | ((y >> 13u) & 0x00010000u) | ((z >> 14u) & 0x00008000u) |
						((x >> 14u) & 0x00004000u) | ((y >> 15u) & 0x00002000u) | ((z >> 16u) & 0x00001000u) |
						((x >> 16u) & 0x00000800u) | ((y >> 17u) & 0x00000400u) | ((z >> 18u) & 0x00000200u) |
						((x >> 18u) & 0x00000100u) | ((y >> 19u) & 0x00000080u) | ((z >> 20u) & 0x00000040u) |
						((x >> 20u) & 0x00000020u) | ((y >> 21u) & 0x00000010u) | ((z >> 22u) & 0x00000008u) |
						((x >> 22u) & 0x00000004u) | ((y >> 23u) & 0x00000002u) | ((z >> 24u) & 0x00000001u);
	return tripleted;
}

fn origin_direction_interleaved_key(origin: vec4<f32>,direction: vec4<f32>) -> u32 {
	let x = bitcast<u32>(origin.x);
	let y = bitcast<u32>(origin.y);
	let z = bitcast<u32>(origin.z);
	let dx = bitcast<u32>(direction.x);
	let dy = bitcast<u32>(direction.y);
	let dz = bitcast<u32>(direction.z);
	let tripleted: u32 = ((x) & 0x80000000u) | ((y >> 1u) & 0x40000000u) | ((z >> 2u) & 0x20000000u) |
						((x >> 5u) & 0x10000000u) | ((y >> 6u) & 0x08000000u) | ((z >> 7u) & 0x04000000u) |
						((x >> 10u) & 0x02000000u) | ((y >> 11u) & 0x01000000u) | ((z >> 12u) & 0x00800000u) |
						((x >> 15u) & 0x00400000u) | ((y >> 16u) & 0x00200000u) | ((z >> 17u) & 0x00100000u) |
						((x >> 20u) & 0x00080000u) | ((y >> 21u) & 0x00040000u) | ((z >> 22u) & 0x00020000u) |
						((x >> 25u) & 0x00010000u) | ((y >> 26u) & 0x00008000u) |
						((dx >> 22u) & 0x00000200u) | ((dy >> 23u) & 0x00000100u) | ((dz >> 24u) & 0x00000080u) |
						((dx >> 27u) & 0x00000040u) | ((dy >> 28u) & 0x00000020u) | ((dz >> 29u) & 0x00000010u);
	return tripleted;
}

//TODO: implement correct key generation codes
@compute @workgroup_size(8,8,1)
fn generate_key32
//fn main
(@builtin(global_invocation_id) gid: vec3<u32>) {
	let id: u32 = scalar_wgid(gid);
	let key: u32 = origin_key(rays[id].origin,rays[id].direction);
	keys32[id] = key;
	block_start_for_radix[id] = 0u;
	prefix_sum_array[id] = array<u32,4>(0u,0u,0u,0u);
	block_sums[id/block_size] = array<u32,4>(0u,0u,0u,0u);
	//atomicStore(&is_ordered[0], 1u);
	//starting_bit[0]= 1u;
	return;
}

//COMPLETE
@compute @workgroup_size(8,8,1)
//fn main
fn check_order_key32
(@builtin(global_invocation_id) gid: vec3<u32>) {
	let id: u32 = scalar_wgid(gid);
	atomicAnd(&is_ordered[0], u32(keys32[id] <= keys32[id + 1u]));
}

@compute @workgroup_size(8,8,1)
fn prefix_sum_step1(@builtin(global_invocation_id) gid: vec3<u32>) {
	let id: u32 = scalar_wgid(gid);
	if (id < number_of_blocks) {
		var zero_count: u32 = 0u;
		var one_count: u32 = 0u;
		var two_count: u32 = 0u;
		var three_count: u32 = 0u;
		let mask: u32 = 3u << (starting_bit[0]);
		let zero_mask: u32 = 0u << (starting_bit[0]);
		let one_mask: u32 = 1u << (starting_bit[0]);
		let two_mask: u32 = 2u << (starting_bit[0]);
		let three_mask: u32 = 3u << (starting_bit[0]);
		for (var i: u32 = 0u; i < block_size; i++) {
			let masked_element: u32 = keys32[id*block_size + i] & mask;
			if (i != 0u && block_start_for_radix[id*block_size+i] != block_start_for_radix[id*block_size+i - 1u]) {
				radix_block_info[block_start_for_radix[id*block_size+i - 1u]][0]=zero_count;
				radix_block_info[block_start_for_radix[id*block_size+i - 1u]][1]=one_count;
				radix_block_info[block_start_for_radix[id*block_size+i - 1u]][2]=two_count;
				radix_block_info[block_start_for_radix[id*block_size+i - 1u]][3]=three_count;
				zero_count = 0u;
				one_count = 0u;
				two_count = 0u;
				three_count = 0u;
			}
			prefix_sum_array[id*block_size+i][0]=zero_count;
			prefix_sum_array[id*block_size+i][1]=one_count;
			prefix_sum_array[id*block_size+i][2]=two_count;
			prefix_sum_array[id*block_size+i][3]=three_count;
			if (masked_element==zero_mask) {
				zero_count++;
			} else if (masked_element==one_mask) {
				one_count++;
			} else if (masked_element==two_mask) {
				two_count++;
			} else {
				three_count++;
			}
		}
		if (block_size * id + block_size >= number_of_elements || block_start_for_radix[id*block_size+block_size]!=block_start_for_radix[id*block_size+block_size - 1u]) {
			radix_block_info[block_start_for_radix[id*block_size+block_size - 1u]][0] = zero_count;
			radix_block_info[block_start_for_radix[id*block_size+block_size - 1u]][1] = one_count;
			radix_block_info[block_start_for_radix[id*block_size+block_size - 1u]][2] = two_count;
			radix_block_info[block_start_for_radix[id*block_size+block_size - 1u]][3] = three_count;
			block_sums[id][0] = 0u;
			block_sums[id][1] = 0u;
			block_sums[id][2] = 0u;
			block_sums[id][3] = 0u;
		} else {
			block_sums[id][0] = zero_count;
			block_sums[id][1] = one_count;
			block_sums[id][2] = two_count;
			block_sums[id][3] = three_count;
		}
	}
}

@compute @workgroup_size(8,8,1)
fn prefix_sum_block_sum(@builtin(global_invocation_id) gid: vec3<u32>) {
	let id: u32 = scalar_wgid(gid);
	if (id == 0u) {
		for (var i: u32 = 1u; i < number_of_blocks; i++) {
			var zero_count: u32 = block_sums[id + i - 1u][0];
			var one_count: u32 = block_sums[id + i - 1u][1];
			var two_count: u32 = block_sums[id + i - 1u][2];
			var three_count: u32 = block_sums[id + i - 1u][3];
			if (block_sums[id+i][0]+block_sums[id+i][1]+block_sums[id+i][2]+block_sums[id+i][3] < block_size) {
				zero_count = 0u;
				one_count = 0u;
				two_count = 0u;
				three_count = 0u;
			}
			block_sums[id+i][0]+=zero_count;
			block_sums[id+i][1]+=one_count;
			block_sums[id+i][2]+=two_count;
			block_sums[id+i][3]+=three_count;
		}
	}
}

@compute @workgroup_size(8,8,1)
fn prefix_sum_step2(@builtin(global_invocation_id) gid: vec3<u32>) {
	let id: u32 = scalar_wgid(gid);
	if (id < number_of_blocks && id != 0u) {
		let mask: u32 = 3u << (starting_bit[0]);
		let zero_mask: u32 = 0u << (starting_bit[0]);
		let one_mask: u32 = 1u << (starting_bit[0]);
		let two_mask: u32 = 2u << (starting_bit[0]);
		let three_mask: u32 = 3u << (starting_bit[0]);
		for (var i: u32 = 0u; i < block_size; i++) {
			if (i != 0u && block_start_for_radix[id*block_size+i] != block_start_for_radix[id*block_size+i - 1u]) {
				radix_block_info[block_start_for_radix[id*block_size+i - 1u]][0]+=block_sums[id - 1u][0];
				radix_block_info[block_start_for_radix[id*block_size+i - 1u]][1]+=block_sums[id - 1u][1];
				radix_block_info[block_start_for_radix[id*block_size+i - 1u]][2]+=block_sums[id - 1u][2];
				radix_block_info[block_start_for_radix[id*block_size+i - 1u]][3]+=block_sums[id - 1u][3];
				block_sums[id - 1u][0] = 0u;
				block_sums[id - 1u][1] = 0u;
				block_sums[id - 1u][2] = 0u;
				block_sums[id - 1u][3] = 0u;
			}
			prefix_sum_array[id*block_size+i][0]+=block_sums[id - 1u][0];
			prefix_sum_array[id*block_size+i][1]+=block_sums[id - 1u][1];
			prefix_sum_array[id*block_size+i][2]+=block_sums[id - 1u][2];
			prefix_sum_array[id*block_size+i][3]+=block_sums[id - 1u][3];
		}
		if (block_size * id + block_size >= number_of_elements || block_start_for_radix[id*block_size+block_size]!=block_start_for_radix[id*block_size+block_size - 1u]) {
			radix_block_info[block_start_for_radix[id*block_size+block_size - 1u]][0] += block_sums[id - 1u][0];
			radix_block_info[block_start_for_radix[id*block_size+block_size - 1u]][1] += block_sums[id - 1u][1];
			radix_block_info[block_start_for_radix[id*block_size+block_size - 1u]][2] += block_sums[id - 1u][2];
			radix_block_info[block_start_for_radix[id*block_size+block_size - 1u]][3] += block_sums[id - 1u][3];

		}
	}
}

@compute @workgroup_size(8,8,1)
fn map_array_key32(@builtin(global_invocation_id) gid: vec3<u32>) {
	let id: u32 = scalar_wgid(gid);
	let mask: u32 = 3u << (starting_bit[0]);
	let zero_mask: u32 = 0u << (starting_bit[0]);
	let one_mask: u32 = 1u << (starting_bit[0]);
	let two_mask: u32 = 2u << (starting_bit[0]);
	let three_mask: u32 = 3u << (starting_bit[0]);
	var sum: u32 = 0u;
	let masked_element: u32 = keys32[id] & mask;
	var index: u32 = 0u;
	if (masked_element==zero_mask) {
		index= 0u;
	}
	if (masked_element==one_mask) {
		index = 1u;
		sum += radix_block_info[block_start_for_radix[id]][0];
	}
	if (masked_element==two_mask) {
		index = 2u;
		sum += radix_block_info[block_start_for_radix[id]][0];
		sum += radix_block_info[block_start_for_radix[id]][1];
	}
	if (masked_element==three_mask) {
		index = 3u;
		sum += radix_block_info[block_start_for_radix[id]][0];
		sum += radix_block_info[block_start_for_radix[id]][1];
		sum += radix_block_info[block_start_for_radix[id]][2];
	}
	block_start_for_radix[id] += sum;
	rays_output[block_start_for_radix[id] + prefix_sum_array[id][index]] = rays[id];
	keys32_out[block_start_for_radix[id] + prefix_sum_array[id][index]] = keys32[id];
	block_start_for_radix_output[block_start_for_radix[id] + prefix_sum_array[id][index]] = block_start_for_radix[id];
	block_sums[id/block_size] = array<u32,4>(0u, 0u, 0u, 0u);
	prefix_sum_array[id] = array<u32,4>(0u, 0u, 0u, 0u);
}
