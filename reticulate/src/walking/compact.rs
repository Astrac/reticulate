use crate::{
    CompactOctree, DenseWalk, DenseWalker, Morton3D, Walk, Walker,
    octree::{CompactOctreeNode, TREE_DEPTH},
};

impl<Data> Walk<Data> for CompactOctree<Data> {
    fn walk(&self, walker: &mut impl Walker<Data>) {
        let mut pending_branches = [0u8; TREE_DEPTH];
        let mut depth: usize = 0;

        let mut current_branch_mask = 1u8;
        let mut current_morton = 0u32;

        for node in self.nodes.iter() {
            let octant = current_branch_mask.trailing_zeros() as u8;
            let node_morton = (current_morton << 3) | octant as u32;

            match node {
                CompactOctreeNode::Branch(mask) => {
                    if depth < TREE_DEPTH {
                        pending_branches[depth] = current_branch_mask;
                        depth += 1;
                    } else {
                        // TODO: Error/warning
                    }

                    current_branch_mask = *mask;
                    current_morton = node_morton;

                    walker.on_branch_start(
                        Morton3D::from_code(node_morton, (depth - 1).into()),
                        *mask,
                    );
                }
                CompactOctreeNode::Leaf(data) => {
                    current_branch_mask &= current_branch_mask - 1;

                    walker.on_leaf(Morton3D::from_code(node_morton, depth.into()), data);
                }
            }

            while current_branch_mask == 0 && depth > 0 {
                depth -= 1;
                let mask = pending_branches[depth];

                let branch_morton = current_morton;
                current_morton >>= 3;

                current_branch_mask = mask & (mask - 1);

                walker.on_branch_end(Morton3D::from_code(branch_morton, depth.into()));
            }
        }
    }
}

impl<Data> DenseWalk<Data> for CompactOctree<Data> {
    fn walk_dense(&self, walker: &mut impl DenseWalker<Data>) {
        let mut pending_branches = [0u8; TREE_DEPTH];
        let mut pending_cursor = [0u8; TREE_DEPTH];
        let mut depth: usize = 0;
        let mut current_branch_mask = 1u8;
        let mut current_morton = 0u32;
        let mut cursor: u8 = 0;

        for node in self.nodes.iter() {
            let octant = current_branch_mask.trailing_zeros() as u8;
            let node_morton = (current_morton << 3) | octant as u32;

            // Any octant strictly between `cursor` and this node's octant
            // was skipped in the mask, i.e. it's empty.
            for empty_octant in cursor..octant {
                let empty_morton = (current_morton << 3) | empty_octant as u32;
                walker.on_empty(Morton3D::from_code(empty_morton, depth.into()));
            }

            match node {
                CompactOctreeNode::Branch(mask) => {
                    if depth < TREE_DEPTH {
                        pending_branches[depth] = current_branch_mask;
                        pending_cursor[depth] = octant + 1;
                        depth += 1;
                    } else {
                        // TODO: Error/warning
                    }
                    current_branch_mask = *mask;
                    current_morton = node_morton;
                    cursor = 0;
                    walker.on_branch_start(
                        Morton3D::from_code(node_morton, (depth - 1).into()),
                        *mask,
                    );
                }
                CompactOctreeNode::Leaf(data) => {
                    current_branch_mask &= current_branch_mask - 1;
                    cursor = octant + 1;
                    walker.on_leaf(Morton3D::from_code(node_morton, depth.into()), data);
                }
            }

            while current_branch_mask == 0 && depth > 0 {
                // Trailing octants (after the last set bit, up to 7) are
                // also empty and must be reported before closing the branch.
                for empty_octant in cursor..8 {
                    let empty_morton = (current_morton << 3) | empty_octant as u32;
                    walker.on_empty(Morton3D::from_code(empty_morton, depth.into()));
                }

                depth -= 1;
                let mask = pending_branches[depth];
                let branch_morton = current_morton;
                current_morton >>= 3;
                current_branch_mask = mask & (mask - 1);
                cursor = pending_cursor[depth];
                walker.on_branch_end(Morton3D::from_code(branch_morton, depth.into()));
            }
        }
    }
}
