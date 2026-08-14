use crate::{coords::Coords, morton::Morton};

#[derive(Clone, Debug)]
pub struct Fixed8Octree<Data> {
    pub nodes: Vec<Fixed8OctreeNode<Data>>,
}

// TODO This should somehow be reflected in Coords (validations/assertions/warnings?)
const TREE_DEPTH: u8 = 6;

// TODO Drop Debug for Data
impl<Data: Clone + PartialEq + std::fmt::Debug> Fixed8Octree<Data> {
    pub fn new() -> Fixed8Octree<Data> {
        Fixed8Octree {
            nodes: vec![Fixed8OctreeNode::leaf(None)],
        }
    }

    pub fn nodes(&self) -> &[Fixed8OctreeNode<Data>] {
        &self.nodes
    }

    pub fn get(&self, coords: Coords) -> Option<&Data> {
        let morton_code: Morton = coords.into();

        log::debug!(
            "Getting {coords:?} (morton = {:0>32b})",
            morton_code.as_u32()
        );

        let mut current_index: usize = 0;
        for depth in 0..TREE_DEPTH {
            let shift = (TREE_DEPTH - depth - 1) * 3;
            let octant = ((morton_code.as_u32() >> shift) & 0b111) as usize;

            log::trace!(
                "Evaluating: depth = {depth}; index = {current_index}; octant: {octant} -> {:?}",
                self.nodes[current_index]
            );

            match &self.nodes[current_index] {
                // TODO Use children mask to skip some traversals
                Fixed8OctreeNode::Branch {
                    children_index,
                    children_mask: _,
                } => {
                    let target = children_index + octant;

                    if depth == TREE_DEPTH - 1 {
                        if let Fixed8OctreeNode::Leaf { data } = &self.nodes[target] {
                            return data.as_ref();
                        } else {
                            panic!(
                                "Unexpected: found branch at depth {} when tree-depth is {}",
                                depth, TREE_DEPTH
                            );
                        }
                    }

                    current_index = *children_index + octant;
                }
                Fixed8OctreeNode::Leaf { data } => {
                    return data.as_ref();
                }
            };
        }

        panic!(
            "Unexpected: the search didn't complete within the expected {} iterations",
            TREE_DEPTH
        );
    }

    pub fn update(&mut self, coords: Coords, data: Option<Data>) {
        let morton_code: Morton = coords.into();

        log::debug!(
            "Updating {coords:?} (morton = {:0>32b})",
            morton_code.as_u32()
        );

        let mut current_index: usize = 0;
        for depth in 0..TREE_DEPTH {
            let shift = (TREE_DEPTH - depth - 1) * 3;
            let octant = ((morton_code.as_u32() >> shift) & 0b111) as usize;

            log::trace!(
                "Evaluating update: depth = {depth}; index = {current_index}; octant: {octant}; current_node: {:?}",
                self.nodes[current_index]
            );

            if let Fixed8OctreeNode::Leaf {
                data: ref leaf_data,
            } = self.nodes[current_index]
            {
                if data == *leaf_data {
                    return;
                }

                let children_index = self.nodes().len();
                let leaf_data = leaf_data.clone();
                let children_mask = if leaf_data.is_some() { 0xFF } else { 0x00 };

                self.nodes[current_index] = Fixed8OctreeNode::Branch {
                    children_index,
                    children_mask,
                };

                for _ in 0..7 {
                    self.nodes.push(Fixed8OctreeNode::leaf(leaf_data.clone()));
                }
                self.nodes.push(Fixed8OctreeNode::leaf(leaf_data));
            };

            if let Fixed8OctreeNode::Branch {
                children_index,
                children_mask,
            } = &mut self.nodes[current_index]
            {
                let target = *children_index + octant;

                if depth == TREE_DEPTH - 1 {
                    log::debug!(
                        "Setting leaf at index {target} - has value: {:?}",
                        data.is_some()
                    );

                    // TODO: Bubble up setting children masks if it changed!!!
                    if data.is_some() {
                        *children_mask = *children_mask | (1 << octant);
                    } else {
                        *children_mask = *children_mask & !(1 << octant);
                    }

                    self.nodes[target] = Fixed8OctreeNode::leaf(data);
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
}

#[derive(Clone, Debug)]
pub enum Fixed8OctreeNode<Data> {
    Branch {
        children_index: usize,
        children_mask: u8,
    },
    Leaf {
        data: Option<Data>,
    },
}

impl<Data> Fixed8OctreeNode<Data> {
    fn leaf(data: Option<Data>) -> Fixed8OctreeNode<Data> {
        Fixed8OctreeNode::Leaf { data }
    }
}
