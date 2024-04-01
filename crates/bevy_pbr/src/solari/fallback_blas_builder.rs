// use bevy::{prelude::*, render::render_resource::ShaderType};
use bevy_math::Vec3;
use bevy_render::{
    mesh::{Indices, Mesh, VertexAttributeValues},
    render_resource::ShaderType,
};
use bevy_utils::tracing::warn;

#[derive(Default, ShaderType, Clone, Copy, Debug)]
pub struct GpuSolariMeshPrimitive {
    pub p1: Vec3,
    // The BLAS builder needs to sort the primitive buffer, so we keep track of the primitive's corresponding triangle id here
    pub corresponding_triangle_id: u32,
    pub p2: Vec3,
    pub _padding1: u32,
    pub p3: Vec3,
    pub _padding2: u32,
}

impl From<SolariMeshPrimitive> for GpuSolariMeshPrimitive {
    fn from(value: SolariMeshPrimitive) -> Self {
        Self {
            p1: value.positions[0],
            corresponding_triangle_id: value.corresponding_triangle_id,
            p2: value.positions[1],
            _padding1: 0,
            p3: value.positions[2],
            _padding2: 0,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct SolariMeshPrimitive {
    pub positions: [Vec3; 3],
    pub centroid: Vec3,
    // The BLAS builder needs to sort the primitive buffer, so we keep track of the primitive's corresponding triangle id here
    pub corresponding_triangle_id: u32,
}

#[derive(Default, ShaderType, Clone, Debug)]
pub struct FallbackBlasNode {
    pub aabb_min: Vec3,
    // Index to child a or to first primitive (triangle).
    pub a_or_first_primitive: u32,
    pub aabb_max: Vec3,
    // > 0 indicates leaf and a_or_tri contains index to first tri. Otherwise a_or_tri contains index to child node a.
    pub primitive_count: u32,
}

pub struct FallbackBlas {
    pub nodes: Vec<FallbackBlasNode>,
    pub primitives: Vec<SolariMeshPrimitive>,
}

pub fn build_fallback_blas(mesh: &Mesh) -> Option<(FallbackBlas, u32)> {
    let mut primitives = match build_mesh_primitives(mesh) {
        Some(primitives) => primitives,
        None => return None,
    };

    // let mut centroids: Vec<Vec3> = vec![];
    // // Calculate triangle bounding box centroids
    // for i in 0..primitives.len() {
    //     let mut bounds_min = Vec3::MAX;
    //     let mut bounds_max = Vec3::MIN;

    //     bounds_min = bounds_min.min(primitives[i].positions[0]);
    //     bounds_min = bounds_min.min(primitives[i].positions[1]);
    //     bounds_min = bounds_min.min(primitives[i].positions[2]);

    //     bounds_max = bounds_max.max(primitives[i].positions[0]);
    //     bounds_max = bounds_max.max(primitives[i].positions[1]);
    //     bounds_max = bounds_max.max(primitives[i].positions[2]);

    //     let center = bounds_min + 0.5 * (bounds_max - bounds_min);
    //     centroids.push(center);
    // }

    let mut nodes: Vec<FallbackBlasNode> = vec![];
    let mut root = FallbackBlasNode::default();
    root.a_or_first_primitive = 0;
    root.primitive_count = primitives.len() as u32;
    calculate_node_aabb(&mut root, &primitives);
    nodes.push(root);

    subdivide(0, &mut nodes, &mut primitives);

    let primitive_count = primitives.len() as u32;
    Some((FallbackBlas { nodes, primitives }, primitive_count))
}

fn build_mesh_primitives(mesh: &Mesh) -> Option<Vec<SolariMeshPrimitive>> {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(VertexAttributeValues::as_float3)
        .unwrap()
        .iter()
        .map(|p| Vec3::from_array(*p))
        .collect::<Vec<Vec3>>();

    let indices: Vec<u32> = match mesh.indices() {
        Some(Indices::U16(values)) => values.iter().map(|v| *v as u32).collect::<Vec<u32>>(),
        Some(Indices::U32(values)) => values.clone(),
        None => {
            warn!("Solari fallback BLAS builder can't use mesh with no index buffer.");
            return None;
        }
    };
    let mut primitives = vec![];
    for primitive_id in 0..(indices.len() / 3) {
        let i_0 = primitive_id * 3;
        let v_0 = indices[i_0] as usize;
        let v_1 = indices[i_0 + 1] as usize;
        let v_2 = indices[i_0 + 2] as usize;
        let positions = [positions[v_0], positions[v_1], positions[v_2]];

        // Calculate triangle bounding box center
        let mut bounds_min = Vec3::MAX;
        let mut bounds_max = Vec3::MIN;

        bounds_min = bounds_min.min(positions[0]);
        bounds_min = bounds_min.min(positions[1]);
        bounds_min = bounds_min.min(positions[2]);
        bounds_max = bounds_max.max(positions[0]);
        bounds_max = bounds_max.max(positions[1]);
        bounds_max = bounds_max.max(positions[2]);

        let centroid = bounds_min + 0.5 * (bounds_max - bounds_min);

        primitives.push(SolariMeshPrimitive {
            positions,
            centroid,
            corresponding_triangle_id: primitive_id as u32,
        });
    }

    Some(primitives)
}

#[derive(Default, Copy, Clone)]
pub struct AABB {
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

pub fn swap_primitives(primitives: &mut Vec<SolariMeshPrimitive>, i0: usize, i1: usize) {
    let primitive_0 = primitives[i0].clone();
    primitives[i0] = primitives[i1].clone();
    primitives[i1] = primitive_0;
}

// pub fn swap<T: Clone>(data: &mut [T], i0: usize, i1: usize) {
//     let val0 = data[i0].clone();
//     data[i0] = data[i1].clone();
//     data[i1] = val0;
// }

pub fn subdivide(
    node_idx: usize,
    nodes: &mut Vec<FallbackBlasNode>,
    primitives: &mut Vec<SolariMeshPrimitive>,
    // centroids: &mut Vec<Vec3>,
) {
    if nodes[node_idx].primitive_count <= 8 {
        return;
    }

    let (axis, split_position, _split_cost) = find_best_split_plane(&nodes[node_idx], primitives);

    // TODO: This should in theory increase performance but makes it worse somehow
    // let no_split_cost = calculate_node_cost(&nodes[node_idx]);
    // if split_cost >= no_split_cost {
    //     return;
    // }

    let mut i = nodes[node_idx].a_or_first_primitive;
    let mut j = i + nodes[node_idx].primitive_count - 1;
    while i <= j {
        if primitives[i as usize].centroid[axis] < split_position {
            i += 1;
        } else {
            // primitives.swap(i as usize, j as usize);
            swap_primitives(primitives, i as usize, j as usize);
            // centroids.swap(i as usize, j as usize);
            j -= 1;
        }
    }

    let a_count = i - nodes[node_idx].a_or_first_primitive;
    // Don't split the nodes[node_idx] if either one of it's children contain no primitives.
    if a_count == 0 || a_count == nodes[node_idx].primitive_count {
        return;
    }

    let mut child_a = FallbackBlasNode::default();
    child_a.a_or_first_primitive = nodes[node_idx].a_or_first_primitive;
    child_a.primitive_count = a_count;
    calculate_node_aabb(&mut child_a, primitives);
    let child_a_index = nodes.len() as u32;
    nodes.push(child_a);

    let mut child_b = FallbackBlasNode::default();
    child_b.a_or_first_primitive = i;
    child_b.primitive_count = nodes[node_idx].primitive_count - a_count;
    calculate_node_aabb(&mut child_b, primitives);
    nodes.push(child_b);

    nodes[node_idx].a_or_first_primitive = child_a_index;
    // Parent nodes[node_idx] is not a leaf, so set prim count to 0.
    nodes[node_idx].primitive_count = 0;

    subdivide(
        nodes[node_idx].a_or_first_primitive as usize,
        nodes,
        primitives,
        // centroids,
    );
    subdivide(
        nodes[node_idx].a_or_first_primitive as usize + 1,
        nodes,
        primitives,
        // centroids,
    );
}

#[derive(Default, Copy, Clone)]
struct Bin {
    bounds: AABB,
    tri_count: u32,
}

// Returns (axis, position, cost)
fn find_best_split_plane(
    node: &FallbackBlasNode,
    primitives: &Vec<SolariMeshPrimitive>,
    // centroids: &Vec<Vec3>,
) -> (usize, f32, f32) {
    let mut best_axis = 0;
    let mut best_position = 0.0;
    let mut best_cost = 1e30;
    for axis in 0..3 {
        let mut bounds_min: f32 = 1e30;
        let mut bounds_max: f32 = -1e30;
        for i in 0..node.primitive_count {
            bounds_min =
                bounds_min.min(primitives[(node.a_or_first_primitive + i) as usize].centroid[axis]);
            bounds_max =
                bounds_max.max(primitives[(node.a_or_first_primitive + i) as usize].centroid[axis]);
        }
        if bounds_min == bounds_max {
            continue;
        }

        // Create bins
        const BIN_COUNT: usize = 20;
        let mut bins: [Bin; BIN_COUNT] = [Bin::default(); BIN_COUNT];
        let bin_size_inv = BIN_COUNT as f32 / (bounds_max - bounds_min);
        for i in 0..node.primitive_count {
            let triangle = &primitives[(node.a_or_first_primitive + i) as usize];
            let bin_idx = (BIN_COUNT - 1).min(
                ((primitives[(node.a_or_first_primitive + i) as usize].centroid[axis] - bounds_min)
                    * bin_size_inv) as usize,
            );
            bins[bin_idx].tri_count += 1;
            bins[bin_idx].bounds.grow_position(triangle.positions[0]);
            bins[bin_idx].bounds.grow_position(triangle.positions[1]);
            bins[bin_idx].bounds.grow_position(triangle.positions[2]);
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
            sum_a += bins[i].tri_count;
            count_a[i] = sum_a;
            box_a.grow_aabb(bins[i].bounds);
            area_a[i] = box_a.area();

            sum_b += bins[BIN_COUNT - 1 - i].tri_count;
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

fn calculate_node_aabb(node: &mut FallbackBlasNode, primitives: &Vec<SolariMeshPrimitive>) {
    node.aabb_min = Vec3::MAX;
    node.aabb_max = Vec3::MIN;
    for i in node.a_or_first_primitive..(node.a_or_first_primitive + node.primitive_count - 1) {
        node.aabb_min = node.aabb_min.min(primitives[i as usize].positions[0]);
        node.aabb_min = node.aabb_min.min(primitives[i as usize].positions[1]);
        node.aabb_min = node.aabb_min.min(primitives[i as usize].positions[2]);

        node.aabb_max = node.aabb_max.max(primitives[i as usize].positions[0]);
        node.aabb_max = node.aabb_max.max(primitives[i as usize].positions[1]);
        node.aabb_max = node.aabb_max.max(primitives[i as usize].positions[2]);
    }
}
