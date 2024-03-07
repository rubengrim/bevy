use bevy_math::{Mat4, Vec3, Vec4};
use bevy_render::render_resource::ShaderType;

use super::fallback_blas_builder::FallbackBlas;

#[derive(Default, ShaderType, Clone, Debug)]
pub struct FallbackTlasNode {
    pub aabb_min: Vec3,
    pub a_or_first_instance: u32,
    pub aabb_max: Vec3,
    pub instance_count: u32,
}

impl FallbackTlasNode {
    fn new_invalid() -> Self {
        // A valid TLAS node can never be an interior node with a child node at index 0.
        Self {
            a_or_first_instance: 0,
            instance_count: 0,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct FallbackTlas {
    pub nodes: Vec<FallbackTlasNode>,
    pub instance_indices: Vec<u32>,
}

#[derive(Default, ShaderType, Clone, Debug)]
pub struct GpuFallbackTlasInstance {
    pub object_world: Mat4,
    pub world_object: Mat4,
    pub primitive_offset: u32,
    pub primitive_count: u32,
    pub blas_node_offset: u32,
}

impl From<FallbackTlasInstance> for GpuFallbackTlasInstance {
    fn from(value: FallbackTlasInstance) -> Self {
        Self {
            object_world: value.object_world,
            world_object: value.world_object,
            primitive_offset: value.primitive_offset,
            primitive_count: value.primitive_count,
            blas_node_offset: value.blas_node_offset,
        }
    }
}

#[derive(Copy, Clone)]
pub struct FallbackTlasInstance {
    pub object_world: Mat4,
    pub world_object: Mat4,
    pub primitive_offset: u32,
    pub primitive_count: u32,
    pub blas_node_offset: u32,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub center: Vec3,
}

impl FallbackTlasInstance {
    pub fn new(
        blas: &FallbackBlas,
        transform: Mat4,
        blas_node_offset: u32,
        primitive_offset: u32,
    ) -> Self {
        let root_node = &blas.nodes[0];
        let b_min = root_node.aabb_min;
        let b_max = root_node.aabb_max;
        let mut b_min_world = Vec3::MAX;
        let mut b_max_world = Vec3::MIN;

        let transform_position = |v: Vec3, m: Mat4| -> Vec3 {
            let homogeneous = m.mul_vec4(Vec4::new(v.x, v.y, v.z, 1.0));
            Vec3::new(homogeneous.x, homogeneous.y, homogeneous.z) / homogeneous.w
        };

        // Find world space positions of all root node corners
        let corner_1 = transform_position(Vec3::new(b_min.x, b_min.y, b_min.z), transform);
        let corner_2 = transform_position(Vec3::new(b_max.x, b_min.y, b_min.z), transform);
        let corner_3 = transform_position(Vec3::new(b_min.x, b_max.y, b_min.z), transform);
        let corner_4 = transform_position(Vec3::new(b_min.x, b_min.y, b_max.z), transform);
        let corner_5 = transform_position(Vec3::new(b_max.x, b_max.y, b_min.z), transform);
        let corner_6 = transform_position(Vec3::new(b_min.x, b_max.y, b_max.z), transform);
        let corner_7 = transform_position(Vec3::new(b_max.x, b_min.y, b_max.z), transform);
        let corner_8 = transform_position(Vec3::new(b_max.x, b_max.y, b_max.z), transform);

        // Calculate bounds of world space root node
        b_min_world = b_min_world.min(corner_1);
        b_min_world = b_min_world.min(corner_2);
        b_min_world = b_min_world.min(corner_3);
        b_min_world = b_min_world.min(corner_4);
        b_min_world = b_min_world.min(corner_5);
        b_min_world = b_min_world.min(corner_6);
        b_min_world = b_min_world.min(corner_7);
        b_min_world = b_min_world.min(corner_8);

        b_max_world = b_max_world.max(corner_1);
        b_max_world = b_max_world.max(corner_2);
        b_max_world = b_max_world.max(corner_3);
        b_max_world = b_max_world.max(corner_4);
        b_max_world = b_max_world.max(corner_5);
        b_max_world = b_max_world.max(corner_6);
        b_max_world = b_max_world.max(corner_7);
        b_max_world = b_max_world.max(corner_8);

        let center = b_min_world + 0.5 * (b_max_world - b_min_world);

        FallbackTlasInstance {
            object_world: transform,
            world_object: transform.inverse(),
            primitive_offset,
            primitive_count: blas.primitives.len() as u32,
            blas_node_offset,
            bounds_min: b_min_world,
            bounds_max: b_max_world,
            center,
        }
    }
}

pub fn build_fallback_tlas(instances: &Vec<FallbackTlasInstance>) -> FallbackTlas {
    if instances.len() == 0 {
        return FallbackTlas {
            nodes: vec![FallbackTlasNode::new_invalid()],
            instance_indices: vec![],
        };
    }

    let mut instance_indices = vec![];
    for i in 0..instances.len() {
        instance_indices.push(i);
    }

    let mut nodes: Vec<FallbackTlasNode> = vec![];
    let mut root = FallbackTlasNode::default();
    root.a_or_first_instance = 0;
    root.instance_count = instances.len() as u32;
    calculate_node_aabb(&mut root, instances, &instance_indices);
    nodes.push(root);

    subdivide(0, &mut nodes, instances, &mut instance_indices);

    let instance_indices = instance_indices
        .iter()
        .map(|i| *i as u32)
        .collect::<Vec<u32>>();

    FallbackTlas {
        nodes,
        instance_indices,
    }
}

pub fn subdivide(
    node_idx: usize,
    nodes: &mut Vec<FallbackTlasNode>,
    instances: &Vec<FallbackTlasInstance>,
    instance_indices: &mut Vec<usize>,
) {
    if nodes[node_idx].instance_count <= 1 {
        return;
    }

    let (axis, split_position, _split_cost) =
        find_best_split_plane(&nodes[node_idx], instances, instance_indices);

    // TODO: This should in theory increase performance but makes it worse somehow
    // let no_split_cost = calculate_node_cost(&nodes[node_idx]);
    // if split_cost >= no_split_cost {
    //     return;
    // }

    let mut i = nodes[node_idx].a_or_first_instance;
    let mut j = i + nodes[node_idx].instance_count - 1;
    while i <= j {
        if instances[instance_indices[i as usize]].center[axis] < split_position {
            i += 1;
        } else {
            instance_indices.swap(i as usize, j as usize);
            j -= 1;
        }
    }

    let a_count = i - nodes[node_idx].a_or_first_instance;
    // Don't split the nodes[node_idx] if either one of it's children contain no primitives.
    if a_count == 0 || a_count == nodes[node_idx].instance_count {
        return;
    }

    let mut child_a = FallbackTlasNode::default();
    child_a.a_or_first_instance = nodes[node_idx].a_or_first_instance;
    child_a.instance_count = a_count;
    calculate_node_aabb(&mut child_a, instances, instance_indices);
    let child_a_index = nodes.len() as u32;
    nodes.push(child_a);

    let mut child_b = FallbackTlasNode::default();
    child_b.a_or_first_instance = i;
    child_b.instance_count = nodes[node_idx].instance_count - a_count;
    calculate_node_aabb(&mut child_b, instances, instance_indices);
    nodes.push(child_b);

    nodes[node_idx].a_or_first_instance = child_a_index;
    // Parent nodes[node_idx] is not a leaf, so set prim count to 0.
    nodes[node_idx].instance_count = 0;

    subdivide(
        nodes[node_idx].a_or_first_instance as usize,
        nodes,
        instances,
        instance_indices,
    );
    subdivide(
        nodes[node_idx].a_or_first_instance as usize + 1,
        nodes,
        instances,
        instance_indices,
    );
}

#[derive(Default, Copy, Clone)]
struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn grow_position(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    pub fn grow_aabb(&mut self, aabb: AABB) {
        self.grow_position(aabb.min);
        self.grow_position(aabb.max);
    }

    pub fn area(&self) -> f32 {
        let e = self.max - self.min;
        e.x * e.y + e.y * e.z + e.z * e.x
    }
}

#[derive(Default, Copy, Clone)]
struct Bin {
    bounds: AABB,
    instance_count: u32,
}

// Returns (axis, position, cost)
fn find_best_split_plane(
    node: &FallbackTlasNode,
    instances: &Vec<FallbackTlasInstance>,
    instance_indices: &Vec<usize>,
) -> (usize, f32, f32) {
    let mut best_axis = 0;
    let mut best_position = 0.0;
    let mut best_cost = 1e30;
    for axis in 0..3 {
        let mut bounds_min: f32 = 1e30;
        let mut bounds_max: f32 = -1e30;
        for i in 0..node.instance_count {
            bounds_min = bounds_min.min(
                instances[instance_indices[(node.a_or_first_instance + i) as usize]].center[axis],
            );
            bounds_max = bounds_max.max(
                instances[instance_indices[(node.a_or_first_instance + i) as usize]].center[axis],
            );
        }
        if bounds_min == bounds_max {
            continue;
        }

        // Create bins
        const BIN_COUNT: usize = 2;
        let mut bins: [Bin; BIN_COUNT] = [Bin::default(); BIN_COUNT];
        let bin_size_inv = BIN_COUNT as f32 / (bounds_max - bounds_min);
        for i in 0..node.instance_count {
            let instance = &instances[instance_indices[(node.a_or_first_instance + i) as usize]];
            let bin_idx =
                (BIN_COUNT - 1).min(((instance.center[axis] - bounds_min) * bin_size_inv) as usize);
            bins[bin_idx].instance_count += 1;
            bins[bin_idx].bounds.grow_position(instance.bounds_min);
            bins[bin_idx].bounds.grow_position(instance.bounds_max);
        }

        // Calculate bin data
        let mut area_a = [0.0; BIN_COUNT - 1];
        let mut area_b = [0.0; BIN_COUNT - 1];
        let mut count_a = [0u32; BIN_COUNT - 1];
        let mut count_b = [0u32; BIN_COUNT - 1];
        let mut box_a = AABB::default();
        let mut box_b = AABB::default();
        let mut sum_a = 0;
        let mut sum_b = 0;
        for i in 0..(BIN_COUNT - 1) {
            sum_a += bins[i].instance_count;
            count_a[i] = sum_a;
            box_a.grow_aabb(bins[i].bounds);
            area_a[i] = box_a.area();

            sum_b += bins[BIN_COUNT - 1 - i].instance_count;
            count_b[BIN_COUNT - 2 - i] = sum_b;
            box_b.grow_aabb(bins[BIN_COUNT - 1 - i].bounds);
            area_b[BIN_COUNT - 2 - i] = box_b.area();
        }

        let bin_size = (bounds_max - bounds_min) / BIN_COUNT as f32;
        for i in 0..(BIN_COUNT - 1) {
            let plane_cost = count_a[i] as f32 * area_a[i] + count_b[i] as f32 * area_b[i];
            if plane_cost < best_cost {
                best_axis = axis;
                best_position = bounds_min + bin_size * (i + 1) as f32;
                best_cost = plane_cost;
            }
        }
    }

    (best_axis, best_position, best_cost)
}

fn calculate_node_aabb(
    node: &mut FallbackTlasNode,
    instances: &Vec<FallbackTlasInstance>,
    instance_indices: &Vec<usize>,
) {
    node.aabb_min = Vec3::MAX;
    node.aabb_max = Vec3::MIN;
    for i in node.a_or_first_instance..(node.a_or_first_instance + node.instance_count) {
        let instance = &instances[instance_indices[i as usize]];
        node.aabb_min = node.aabb_min.min(instance.bounds_min);
        node.aabb_max = node.aabb_max.max(instance.bounds_max);
    }
}
