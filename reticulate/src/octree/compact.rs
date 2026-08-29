use crate::{Depth, Morton3D, Walk, Walker, octree::Fixed8Octree};

#[derive(Clone, Debug, Default)]
pub struct CompactOctree<Data> {
    pub nodes: Vec<CompactOctreeNode<Data>>,
}

impl<Data: Copy + Default + PartialEq> CompactOctree<Data> {
    pub fn from_fixed8(octree: &Fixed8Octree<Data>) -> CompactOctree<Data> {
        let mut walker = ToCompactWalker::default();
        octree.walk(&mut walker);

        walker.compact
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum CompactOctreeNode<Data> {
    Branch(u8),
    Leaf(Data),
}

enum UniformState<Data> {
    Unset,
    Uniform(Data),
    NonUniform,
}

struct Frame<Data> {
    start: usize,
    seen_mask: u8,
    state: UniformState<Data>,
}

// TODO -  Remove Vec, use stack-array
#[derive(Default)]
pub struct ToCompactWalker<Data> {
    pub compact: CompactOctree<Data>,
    stack: Vec<Frame<Data>>,
}

impl<Data: Copy + PartialEq> ToCompactWalker<Data> {
    fn note_child(&mut self, octant: u8, data: Option<Data>) {
        if let Some(frame) = self.stack.last_mut() {
            frame.seen_mask |= 1 << octant;
            frame.state = match (&frame.state, data) {
                (UniformState::NonUniform, _) | (_, None) => UniformState::NonUniform,
                (UniformState::Unset, Some(d)) => UniformState::Uniform(d),
                (UniformState::Uniform(prev), Some(d)) if *prev == d => UniformState::Uniform(d),
                (UniformState::Uniform(_), Some(_)) => UniformState::NonUniform,
            };
        }
    }
}

impl<Data: Copy + PartialEq> Walker<Data> for ToCompactWalker<Data> {
    fn on_branch_start(&mut self, _depth: Depth, _morton: Morton3D, _children_mask: u8) {
        let start = self.compact.nodes.len();
        self.compact.nodes.push(CompactOctreeNode::Branch(0));
        self.stack.push(Frame {
            start,
            seen_mask: 0,
            state: UniformState::Unset,
        });
    }

    fn on_branch_end(&mut self, _depth: Depth, morton: Morton3D) {
        let frame = self
            .stack
            .pop()
            .expect("on_branch_end without matching on_branch_start");

        let collapse = match frame.state {
            UniformState::Uniform(data) if frame.seen_mask == 0xFF => Some(data),
            _ => None,
        };

        if let Some(data) = collapse {
            self.compact.nodes.truncate(frame.start);
            self.compact.nodes.push(CompactOctreeNode::Leaf(data));
            self.note_child(morton.octant(), Some(data));
        } else {
            self.compact.nodes[frame.start] = CompactOctreeNode::Branch(frame.seen_mask);
            self.note_child(morton.octant(), None);

            if frame.seen_mask == 0 {
                self.compact.nodes.truncate(frame.start);
            }
        }
    }

    fn on_leaf(&mut self, _depth: Depth, morton: Morton3D, data: &Data) {
        self.compact.nodes.push(CompactOctreeNode::Leaf(*data));
        self.note_child(morton.octant(), Some(*data));
    }
}
