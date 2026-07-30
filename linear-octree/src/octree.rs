use crate::{Coords, Morton};

#[derive(Clone, Debug)]
pub struct Octree<Data> {
    nodes: Vec<OctreeNode<Data>>,
}

// TODO: This should somehow be reflected in Coords (validations/assertions/warnings?)
const TREE_DEPTH: u8 = 6;

// TODO: Drop Debug for Data
impl<Data: Clone + PartialEq + std::fmt::Debug> Octree<Data> {
    pub fn new() -> Octree<Data> {
        Octree {
            nodes: vec![OctreeNode::leaf(None)],
        }
    }

    pub fn nodes(&self) -> &[OctreeNode<Data>] {
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
                OctreeNode::Branch { children_index } => {
                    let target = children_index + octant;

                    if depth == TREE_DEPTH - 1 {
                        if let OctreeNode::Leaf { data } = &self.nodes[target] {
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
                OctreeNode::Leaf { data } => {
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
                "Evaluating: depth = {depth}; index = {current_index}; octant: {octant}; current_node: {:?}",
                self.nodes[current_index]
            );

            if let OctreeNode::Leaf {
                data: ref leaf_data,
            } = self.nodes[current_index]
            {
                if data == *leaf_data {
                    return;
                }

                let children_index = self.nodes().len();
                let leaf_data = leaf_data.clone();

                self.nodes[current_index] = OctreeNode::Branch { children_index };

                for _ in 0..7 {
                    self.nodes.push(OctreeNode::leaf(leaf_data.clone()));
                }
                self.nodes.push(OctreeNode::leaf(leaf_data));
            };

            if let OctreeNode::Branch { children_index } = self.nodes[current_index] {
                let target = children_index + octant;

                if depth == TREE_DEPTH - 1 {
                    self.nodes[target] = OctreeNode::leaf(data);
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
pub enum OctreeNode<Data> {
    Branch { children_index: usize },
    Leaf { data: Option<Data> },
}

impl<Data> OctreeNode<Data> {
    fn leaf(data: Option<Data>) -> OctreeNode<Data> {
        OctreeNode::Leaf { data }
    }
}
