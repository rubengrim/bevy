use bevy_math::{
    bounding::{Aabb3d, BoundingVolume},
    Vec3,
};

pub struct BvhPrimitive {
    pub bounds: Aabb3d,
    // Store centroid explicitly to avoid recalculation
    pub centroid: Vec3,
    pub primitive_id: u32,
}

#[derive(Clone)]
pub struct SbvhNode {
    pub bounds: Aabb3d,
    pub child_a_idx: u32,
    pub first_primitive: u32,
    pub primitive_count: u32,
}

impl Default for SbvhNode {
    fn default() -> Self {
        Self {
            bounds: Aabb3d {
                min: Vec3::MAX,
                max: Vec3::MIN,
            },
            child_a_idx: 0,
            first_primitive: 0,
            primitive_count: 0,
        }
    }
}

pub struct Sbvh {
    pub nodes: Vec<SbvhNode>,
    pub primitive_indices: Vec<u32>,
}

pub fn build_sbvh(mut primitives: Vec<BvhPrimitive>) -> Sbvh {
    let mut nodes: Vec<SbvhNode> = vec![];
    let mut primitive_indices = Vec::from_iter(0..primitives.len());

    let mut root = SbvhNode::default();
    root.first_primitive = 0;
    root.primitive_count = primitives.len() as u32;
    calculate_node_aabb(&mut root, &primitives, &primitive_indices);
    nodes.push(root);

    subdivide(0, &mut nodes, &mut primitives, &mut primitive_indices);

    Sbvh {
        nodes,
        primitive_indices: primitive_indices.iter().map(|i| *i as u32).collect(),
    }
}

fn subdivide(
    node_idx: usize,
    nodes: &mut Vec<SbvhNode>,
    primitives: &Vec<BvhPrimitive>,
    primitive_indices: &mut Vec<usize>,
) {
    if nodes[node_idx].primitive_count <= 8 {
        return;
    }

    // Calculate bounds for centroids of all the primitives contained by the node
    let mut centroid_bounds_min = Vec3::MAX;
    let mut centroid_bounds_max = Vec3::MIN;
    for i in nodes[node_idx].first_primitive
        ..(nodes[node_idx].first_primitive + nodes[node_idx].primitive_count - 1)
    {
        centroid_bounds_min =
            centroid_bounds_min.min(primitives[primitive_indices[i as usize]].centroid);
        centroid_bounds_max =
            centroid_bounds_max.max(primitives[primitive_indices[i as usize]].centroid);
    }
    // Find the axis with the largest extent
    let extent = centroid_bounds_max - centroid_bounds_min;
    let dim: usize;
    if extent.x > extent.y && extent.x > extent.z {
        dim = 0;
    } else if extent.y > extent.z {
        dim = 1;
    } else {
        dim = 2;
    }

    // Don't split the node if the centroids are all the same
    if centroid_bounds_min[dim] == centroid_bounds_max[dim] {
        return;
    }

    let split_position = centroid_bounds_min[dim] + 0.5 * extent[dim];

    // Partition the primitives
    let mut i = nodes[node_idx].first_primitive;
    let mut j = i + nodes[node_idx].primitive_count - 1;
    while i <= j {
        if primitives[primitive_indices[i as usize]].centroid[dim] < split_position {
            i += 1;
        } else {
            primitive_indices.swap(i as usize, j as usize);
            j -= 1;
        }
    }

    // Number of primitives contained in child a
    let a_count = i - nodes[node_idx].first_primitive;
    // Don't split the nodes[node_idx] if either one of it's children contain no primitives
    if a_count == 0 || a_count == nodes[node_idx].primitive_count {
        return;
    }

    // Create node children
    let mut child_a = SbvhNode::default();
    child_a.first_primitive = nodes[node_idx].first_primitive;
    child_a.primitive_count = a_count;
    calculate_node_aabb(&mut child_a, primitives, primitive_indices);
    let child_a_idx = nodes.len() as u32;
    nodes.push(child_a);

    let mut child_b = SbvhNode::default();
    child_b.first_primitive = i;
    child_b.primitive_count = nodes[node_idx].primitive_count - a_count;
    calculate_node_aabb(&mut child_b, primitives, primitive_indices);
    nodes.push(child_b);

    // Set parent to be an interior node
    nodes[node_idx].child_a_idx = child_a_idx;
    nodes[node_idx].primitive_count = 0;

    // Subdivide children
    subdivide(child_a_idx as usize, nodes, primitives, primitive_indices);
    subdivide(
        child_a_idx as usize + 1,
        nodes,
        primitives,
        primitive_indices,
    );
}

fn calculate_node_aabb(
    node: &mut SbvhNode,
    primitives: &Vec<BvhPrimitive>,
    primitive_indices: &Vec<usize>,
) {
    for i in node.first_primitive..(node.first_primitive + node.primitive_count - 1) {
        node.bounds
            .merge(&primitives[primitive_indices[i as usize]].bounds);
    }
}
