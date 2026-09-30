use std::fmt::Debug;

use crate::{
    Fixed8Octree, Morton3D, Walk, Walker,
    octree::{Fixed8OctreeNode, TREE_DEPTH},
    walking::{DenseWalk, DenseWalker},
};

impl<Data> Walk<Data> for Fixed8Octree<Data> {
    fn walk(&self, walker: &mut impl Walker<Data>) {
        let mut pending_branches = [0u8; TREE_DEPTH];
        let mut jumps = [0usize; TREE_DEPTH + 1];
        let mut depth: usize = 0;
        let mut current_branch_mask = 0b11111111_u8;
        let mut current_morton = 0u32;
        let mut done = self.nodes.is_empty();

        while !done {
            let octant = current_branch_mask.trailing_zeros() as u8;
            let node_morton = (current_morton << 3) | octant as u32;
            let node = &self.nodes[jumps[depth]];

            match node {
                Fixed8OctreeNode::Branch {
                    children_index,
                    children_mask: _,
                } => {
                    if depth < TREE_DEPTH {
                        pending_branches[depth] = current_branch_mask;
                        jumps[depth] += 1;
                        depth += 1;
                    } else {
                        // TODO: Error/warning
                    }

                    current_branch_mask = 0b11111111_u8;
                    current_morton = node_morton;
                    jumps[depth] = *children_index;

                    walker.on_branch_start(
                        Morton3D::from_code(node_morton, (depth - 1).into()),
                        current_branch_mask,
                    );
                }
                Fixed8OctreeNode::Leaf { data } => {
                    current_branch_mask &= current_branch_mask - 1;

                    if let Some(data) = data {
                        walker.on_leaf(Morton3D::from_code(node_morton, depth.into()), data);
                    }

                    if depth != 0 {
                        jumps[depth] += 1;
                    } else {
                        return;
                    }
                }
            }

            while current_branch_mask == 0 && depth > 0 {
                depth -= 1;
                let mask = pending_branches[depth];

                let branch_morton = current_morton;
                current_morton >>= 3;

                current_branch_mask = mask & (mask - 1);

                if depth == 0 {
                    done = true;
                }

                walker.on_branch_end(Morton3D::from_code(branch_morton, depth.into()));
            }
        }
    }
}

impl<Data: Debug> DenseWalk<Data> for Fixed8Octree<Data> {
    fn walk_dense(&self, walker: &mut impl DenseWalker<Data>) {
        let mut pending_branches = [0u8; TREE_DEPTH];
        let mut jumps = [0usize; TREE_DEPTH + 1];
        let mut depth: usize = 0;
        let mut current_branch_mask = 0b11111111_u8;
        let mut current_morton = 0u32;
        let mut done = false;

        while !done {
            let octant = current_branch_mask.trailing_zeros() as u8;
            let node_morton = (current_morton << 3) | octant as u32;
            let node = &self.nodes[jumps[depth]];

            match node {
                Fixed8OctreeNode::Branch {
                    children_index,
                    children_mask: _,
                } => {
                    if depth < TREE_DEPTH {
                        pending_branches[depth] = current_branch_mask;
                        jumps[depth] += 1;
                        depth += 1;
                    } else {
                        // TODO: Error/warning
                    }

                    current_branch_mask = 0b11111111_u8;
                    current_morton = node_morton;
                    jumps[depth] = *children_index;

                    walker.on_branch_start(
                        Morton3D::from_code(node_morton, (depth - 1).into()),
                        current_branch_mask,
                    );
                }
                Fixed8OctreeNode::Leaf { data } => {
                    current_branch_mask &= current_branch_mask - 1;

                    if let Some(data) = data {
                        walker.on_leaf(Morton3D::from_code(node_morton, depth.into()), data);
                    } else {
                        walker.on_empty(Morton3D::from_code(node_morton, depth.into()));
                    }

                    if depth != 0 {
                        jumps[depth] += 1;
                    } else {
                        return;
                    }
                }
            }

            while current_branch_mask == 0 && depth > 0 {
                depth -= 1;
                let mask = pending_branches[depth];

                let branch_morton = current_morton;
                current_morton >>= 3;

                current_branch_mask = mask & (mask - 1);

                if depth == 0 {
                    done = true;
                }

                walker.on_branch_end(Morton3D::from_code(branch_morton, depth.into()));
            }
        }
    }
}
