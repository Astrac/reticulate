use crate::{Fixed8Octree, Fixed8OctreeNode};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CompactOctree<Data> {
    pub nodes: Vec<CompactOctreeNode<Data>>,
}

#[allow(dead_code)]
impl<Data: Clone + std::fmt::Debug + PartialEq> CompactOctree<Data> {
    pub fn from_fixed8(source: Fixed8Octree<Data>) -> CompactOctree<Data> {
        let nodes = source.nodes();
        if nodes.is_empty() {
            return CompactOctree { nodes: Vec::new() };
        }

        // Action states for our explicit traversal stack
        enum Action {
            Process(usize),
            Assemble { children_index: usize },
        }

        // Main traversal stack
        let mut stack = vec![Action::Process(0)];

        // Temporary storage for completed subtrees
        // Each entry is a flat sequence of compact nodes representing a valid subtree
        let mut subtrees: Vec<Vec<CompactOctreeNode<Data>>> = Vec::new();

        while let Some(action) = stack.pop() {
            match action {
                Action::Process(index) => {
                    match &nodes[index] {
                        Fixed8OctreeNode::Leaf { data } => {
                            if let Some(val) = data {
                                subtrees.push(vec![CompactOctreeNode::Leaf(val.clone())]);
                            } else {
                                // Push an empty vector to signal an empty leaf slot
                                subtrees.push(Vec::new());
                            }
                        }
                        Fixed8OctreeNode::Branch { children_index, children_mask: _ } => {
                            // Schedule the assembly step for after the children are processed
                            stack.push(Action::Assemble {
                                children_index: *children_index,
                            });
                            // Schedule all 8 children to be processed (LIFO order: 7 down to 0)
                            for i in (0..8).rev() {
                                stack.push(Action::Process(children_index + i));
                            }
                        }
                    }
                }
                Action::Assemble { .. } => {
                    // Pop the 8 processed children subtrees from the back of the accumulator
                    let mut children = subtrees.split_off(subtrees.len() - 8);

                    let mut mask = 0u8;
                    let mut total_nodes = 0;

                    // Calculate mask and count non-empty children
                    for i in 0..8 {
                        if !children[i].is_empty() {
                            mask |= 1 << i;
                            total_nodes += children[i].len();
                        }
                    }

                    // Attempt coalescing: must have 8 valid children, and all must be identical leaves
                    let mut coalesced = false;
                    if mask == 0xFF && total_nodes == 8 {
                        if let Some(CompactOctreeNode::Leaf(first_val)) = children[0].first() {
                            let all_identical = children.iter().all(|child| match child.first() {
                                Some(CompactOctreeNode::Leaf(val)) => val == first_val,
                                _ => false,
                            });

                            if all_identical {
                                subtrees.push(vec![CompactOctreeNode::Leaf(first_val.clone())]);
                                coalesced = true;
                            }
                        }
                    }

                    if !coalesced {
                        if mask == 0 {
                            // All children were empty; push an empty slot to prune this branch
                            subtrees.push(Vec::new());
                        } else {
                            // Build the uncoalesced branch flat sequence
                            let mut branch_layout = Vec::with_capacity(1 + total_nodes);
                            branch_layout.push(CompactOctreeNode::Branch(mask));
                            for child in children.iter_mut() {
                                branch_layout.append(child);
                            }
                            subtrees.push(branch_layout);
                        }
                    }
                }
            }
        }

        // The final remaining subtree item is our fully built, flattened tree layout
        CompactOctree {
            nodes: subtrees.pop().unwrap_or_default(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum CompactOctreeNode<Data> {
    Branch(u8),
    Leaf(Data),
}
