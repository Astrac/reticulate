use crate::{Coords, Morton};

#[derive(Clone, Debug)]
pub struct Octree<Data> {
    nodes: Vec<Node<Data>>,
}

// TODO: This should somehow be reflected in Coords (validations/assertions/warnings?)
const TREE_DEPTH: u8 = 6;

// TODO: Drop Debug for Data
impl<Data: Clone + PartialEq + std::fmt::Debug> Octree<Data> {
    pub fn new() -> Octree<Data> {
        Octree {
            nodes: vec![Node::leaf(None)],
        }
    }

    pub fn nodes(&self) -> &[Node<Data>] {
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
                Node::Branch { children_index } => {
                    let target = children_index + octant;

                    if depth == TREE_DEPTH - 1 {
                        if let Node::Leaf { data } = &self.nodes[target] {
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
                Node::Leaf { data } => {
                    return data.as_ref();
                }
            };
        }

        panic!(
            "Unexpected: the search didn't complete within the expected {} iterations",
            TREE_DEPTH
        );
    }

    pub fn update(&mut self, coords: Coords, data: Data)
    where
        Data: std::fmt::Debug,
    {
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

            if let Node::Leaf {
                data: ref leaf_data,
            } = self.nodes[current_index]
            {
                let children_index = self.nodes().len();
                let data = leaf_data.clone();

                self.nodes[current_index] = Node::Branch { children_index };

                for _ in 0..7 {
                    self.nodes.push(Node::leaf(data.clone()));
                }
                self.nodes.push(Node::leaf(data));
            };

            if let Node::Branch { children_index } = self.nodes[current_index] {
                let target = children_index + octant;

                if depth == TREE_DEPTH - 1 {
                    self.nodes[target] = Node::leaf(Some(data));
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
pub enum Node<Data> {
    Branch { children_index: usize },
    Leaf { data: Option<Data> },
}

impl<Data> Node<Data> {
    fn leaf(data: Option<Data>) -> Node<Data> {
        Node::Leaf { data }
    }
}
