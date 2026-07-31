use crate::{BlockFaces, CompactOctree, Morton};

// 2^6^3 = 2^18 cubes -> 2^18 bit for each cube solid or not

// If I carve a block I'm 100% sure that all the faces of its
// neighbouring blocks it are exposed.
//
//    - Find all the empty space and subtract it from the mesh
//    - Need to define a generic traversal function for octree!
// Two steps:
//    - Determine biggest volumes possible traversing octree
//      * Or maybe better, compress octree?

pub fn draw_octree(octree: &CompactOctree<()>) -> Vec<BlockFaces> {
    let mut faces = Vec::new();
    let nodes = &octree.nodes;

    if nodes.is_empty() {
        return faces;
    }

    let mut pending_indices: Vec<(usize, u8, Morton)> = Vec::new();
    pending_indices.push((0, 0, Morton::from(0)));

    while !pending_indices.is_empty() {
        let Some((index, depth, morton)) = pending_indices.pop() else {
            unreachable!()
        };

        let node = &nodes[index];

        match node {
            crate::CompactOctreeNode::Branch(mask) => {
                let mut mask_track = *mask;
                for i in 0..(mask.count_ones()) {
                    let octant = mask_track.trailing_zeros();
                    mask_track = mask_track & !(1 << octant);
                    let child_morton = Morton::from((morton.as_u32() << 3) | octant);
                    pending_indices.push((index + i as usize + 1, depth + 1, child_morton));
                }
            }
            crate::CompactOctreeNode::Leaf(_) => faces.push(BlockFaces {
                depth,
                coords: morton.into(),
                face_mask: 0b111111,
            }),
        }
    }

    faces
}
