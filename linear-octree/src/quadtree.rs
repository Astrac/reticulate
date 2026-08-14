use smallvec::SmallVec;

use crate::morton::Morton;

#[allow(dead_code)]
const TREE_DEPTH: usize = 6;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Fixed4QuadtreeNode<Data> {
    Branch {
        children_index: usize,
        children_mask: u8,
    },
    Leaf {
        data: Option<Data>,
    },
}

impl<Data> Fixed4QuadtreeNode<Data> {
    fn leaf(data: Option<Data>) -> Fixed4QuadtreeNode<Data> {
        Fixed4QuadtreeNode::Leaf { data }
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Fixed4Quadtree<Data> {
    nodes: SmallVec<[Fixed4QuadtreeNode<Data>; 32]>,
}

#[allow(dead_code)]
impl<Data: PartialEq + Clone> Fixed4Quadtree<Data> {
    pub fn update(&mut self, morton: Morton, data: Option<Data>) {
        // TODO: implement
        let mut current_index: usize = 0;
        for depth in 0..TREE_DEPTH {
            let shift = (TREE_DEPTH - depth - 1) * 2;
            let quadrant = ((morton.as_u32() >> shift) & 0b11) as usize;

            if let Fixed4QuadtreeNode::Leaf {
                data: ref leaf_data,
            } = self.nodes[current_index]
            {
                if data == *leaf_data {
                    return;
                }

                let children_index = self.nodes.len();
                let leaf_data = leaf_data.clone();
                let children_mask = if leaf_data.is_some() { 0xF } else { 0x0 };

                self.nodes[current_index] = Fixed4QuadtreeNode::Branch {
                    children_index,
                    children_mask,
                };

                for _ in 0..3 {
                    self.nodes.push(Fixed4QuadtreeNode::leaf(leaf_data.clone()));
                }
                self.nodes.push(Fixed4QuadtreeNode::leaf(leaf_data));
            };

            if let Fixed4QuadtreeNode::Branch {
                children_index,
                children_mask,
            } = &mut self.nodes[current_index]
            {
                let target = *children_index + quadrant;

                if depth == TREE_DEPTH - 1 {
                    log::debug!(
                        "Setting leaf at index {target} - has value: {:?}",
                        data.is_some()
                    );

                    // TODO: Bubble up setting children masks if it changed!!!
                    if data.is_some() {
                        *children_mask = *children_mask | (1 << quadrant);
                    } else {
                        *children_mask = *children_mask & !(1 << quadrant);
                    }

                    self.nodes[target] = Fixed4QuadtreeNode::leaf(data);
                    return;
                }

                current_index = target;
            } else {
                panic!("Unexpected: Leaf should have been overwritten at this point")
            }
        }

        panic!(
            "Unexpected: the update didn't complete within the expected {} iterations",
            TREE_DEPTH
        );
    }

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
                Fixed4QuadtreeNode::Branch {
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
                Fixed4QuadtreeNode::Leaf { data } => {
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

pub trait Walker<Data> {
    fn on_branch_start(&mut self, depth: u8, morton: Morton, children_mask: u8);
    fn on_branch_end(&mut self, depth: u8, morton: Morton);
    fn on_leaf(&mut self, depth: u8, morton: Morton, data: &Data);
}
