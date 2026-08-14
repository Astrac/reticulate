use crate::{morton_3d::Morton3D, octree::*};

pub trait Walker<Data> {
    fn on_branch_start(&mut self, depth: u8, morton: Morton3D, children_mask: u8);
    fn on_branch_end(&mut self, depth: u8, morton: Morton3D);
    fn on_leaf(&mut self, depth: u8, morton: Morton3D, data: &Data);
}

pub trait Walk<Data> {
    fn walk(&self, walker: &mut impl Walker<Data>);
}

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

// TODO: Optimize
impl<Data> Walk<Data> for Fixed8Octree<Data> {
    fn walk(&self, walker: &mut impl Walker<Data>) {
        let mut current_morton = 0u32;
        let mut remaining_children = 1u8;
        let mut open_branches = Vec::new();
        let mut pending = Vec::new();
        pending.push((0, 0));

        while !pending.is_empty() {
            let Some((node_index, octant)) = pending.pop() else {
                unreachable!()
            };

            let node = &self.nodes[node_index];
            let node_morton = (current_morton << 3) | octant as u32;

            match node {
                Fixed8OctreeNode::Branch {
                    children_index,
                    children_mask,
                } => {
                    for i in 0..8 {
                        pending.push((children_index + i, i as u8));
                    }

                    open_branches.push(remaining_children);
                    remaining_children = 8;
                    current_morton = node_morton;
                    walker.on_branch_start(
                        open_branches.len() as u8,
                        node_morton.into(),
                        *children_mask,
                    );
                }
                Fixed8OctreeNode::Leaf { data } => {
                    remaining_children -= 1;
                    if let Some(data) = &data {
                        walker.on_leaf(open_branches.len() as u8, node_morton.into(), data);
                    }
                }
            }

            while remaining_children == 0 {
                if let Some(remaining) = open_branches.pop() {
                    current_morton = current_morton >> 3;
                    walker.on_branch_end(open_branches.len() as u8, current_morton.into());
                    remaining_children = remaining - 1;
                } else {
                    // TODO Proper error checking here
                    break;
                }
            }
        }
    }
}
