use crate::{CompactOctree, Walk, Walker, octree::CompactOctreeNode};

impl<Data> Walk<Data> for CompactOctree<Data> {
    fn walk(&self, walker: &mut impl Walker<Data>) {
        const MAX_DEPTH: usize = 6;
        let mut pending_branches = [0u8; MAX_DEPTH];
        let mut depth: usize = 0;

        let mut current_branch_mask = 1u8;
        let mut current_morton = 0u32;

        for node in self.nodes.iter() {
            let octant = current_branch_mask.trailing_zeros() as u8;
            let node_morton = (current_morton << 3) | octant as u32;

            match node {
                CompactOctreeNode::Branch(mask) => {
                    if depth < MAX_DEPTH {
                        pending_branches[depth] = current_branch_mask;
                        depth += 1;
                    } else {
                        // TODO: Error/warning
                    }

                    current_branch_mask = *mask;
                    current_morton = node_morton;

                    walker.on_branch_start((depth - 1) as u8, node_morton.into(), *mask);
                }
                CompactOctreeNode::Leaf(data) => {
                    current_branch_mask &= current_branch_mask - 1;

                    walker.on_leaf(depth as u8, node_morton.into(), data);
                }
            }

            while current_branch_mask == 0 && depth > 0 {
                depth -= 1;
                let mask = pending_branches[depth];

                let branch_morton = current_morton;
                current_morton >>= 3;

                current_branch_mask = mask & (mask - 1);

                walker.on_branch_end(depth as u8, branch_morton.into());
            }
        }
    }
}
