use crate::{BlockFaces, CompactOctree, CompactOctreeNode, Morton};

pub fn draw_octree(octree: &CompactOctree<()>) -> Vec<BlockFaces> {
    let mut faces = Vec::new();
    let nodes = &octree.nodes;

    if nodes.is_empty() {
        return faces;
    }

    // Stack tracks: (remaining_branch_mask, original_branch_mask, face_mask_inherited_from_parent)
    let mut pending_branches: Vec<(u8, u8, u8)> = Vec::new();

    let mut current_branch_mask = 1u8;
    let mut original_branch_mask = 1u8;
    let mut current_inherited_faces = 0b111111u8; // Root starts completely unculled
    let mut current_morton = 0u32;

    for node in nodes.iter() {
        let octant = current_branch_mask.trailing_zeros() as u8;
        let node_morton = (current_morton << 3) | octant as u32;

        // Calculate the culling for this specific octant based on its siblings in the current branch
        let mut node_face_mask = current_inherited_faces;

        // X-Axis Culling (Bit 0 of octant)
        if (octant & 1) == 0 {
            if (original_branch_mask & (1 << (octant + 1))) != 0 {
                node_face_mask &= !(1 << 0); // Cull -X face
            }
        } else {
            if (original_branch_mask & (1 << (octant - 1))) != 0 {
                node_face_mask &= !(1 << 1); // Cull +X face
            }
        }

        // Y-Axis Culling (Bit 1 of octant)
        if (octant & 2) == 0 {
            if (original_branch_mask & (1 << (octant + 2))) != 0 {
                node_face_mask &= !(1 << 2); // Cull -Y face
            }
        } else {
            if (original_branch_mask & (1 << (octant - 2))) != 0 {
                node_face_mask &= !(1 << 3); // Cull +Y face
            }
        }

        // Z-Axis Culling (Bit 2 of octant)
        if (octant & 4) == 0 {
            if (original_branch_mask & (1 << (octant + 4))) != 0 {
                node_face_mask &= !(1 << 4); // Cull -Z face
            }
        } else {
            if (original_branch_mask & (1 << (octant - 4))) != 0 {
                node_face_mask &= !(1 << 5); // Cull +Z face
            }
        }

        match node {
            CompactOctreeNode::Branch(mask) => {
                // Save current state before descending into the new branch
                pending_branches.push((current_branch_mask, original_branch_mask, current_inherited_faces));

                current_branch_mask = *mask;
                original_branch_mask = *mask;
                current_inherited_faces = node_face_mask; // Children inherit this node's faces
                current_morton = node_morton;
            }
            CompactOctreeNode::Leaf(_) => {
                current_branch_mask &= current_branch_mask - 1;
                let depth = pending_branches.len() as u8;

                // Only emit the face mesh if it has at least one visible external face
                if node_face_mask != 0 {
                    faces.push(BlockFaces {
                        depth,
                        coords: Morton::from(node_morton).into(),
                        face_mask: node_face_mask,
                    });
                }
            }
        }

        // Ascend the tree levels if the current branch has been fully iterated
        while current_branch_mask == 0 {
            if let Some((b_rem, b_orig, faces_inh)) = pending_branches.pop() {
                current_branch_mask = b_rem & (b_rem - 1);
                original_branch_mask = b_orig;
                current_inherited_faces = faces_inh;
                current_morton = current_morton >> 3;
            } else {
                break;
            }
        }
    }

    faces
}
