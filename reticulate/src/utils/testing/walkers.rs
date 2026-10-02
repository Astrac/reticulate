use std::collections::BTreeMap;

use crate::{DenseWalker, Morton3D, TREE_DEPTH, Walker};

#[derive(Debug, Clone, PartialEq)]
pub enum WalkEvent<Data> {
    BranchStart { morton: Morton3D, mask: u8 },
    BranchEnd { morton: Morton3D },
    Leaf { morton: Morton3D, data: Data },
    Empty { morton: Morton3D },
}

/// A walker that records every callback, for both sparse and dense walks.
#[derive(Debug)]
pub struct RecordingWalker<Data> {
    pub events: Vec<WalkEvent<Data>>,
}

impl<Data> Default for RecordingWalker<Data> {
    fn default() -> Self {
        RecordingWalker { events: Vec::new() }
    }
}

impl<Data: Clone> RecordingWalker<Data> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self, pred: impl Fn(&WalkEvent<Data>) -> bool) -> usize {
        self.events.iter().filter(|e| pred(e)).count()
    }

    pub fn branch_starts(&self) -> usize {
        self.count(|e| matches!(e, WalkEvent::BranchStart { .. }))
    }

    pub fn branch_ends(&self) -> usize {
        self.count(|e| matches!(e, WalkEvent::BranchEnd { .. }))
    }

    pub fn leaves(&self) -> Vec<(Morton3D, Data)> {
        self.events
            .iter()
            .filter_map(|e| match e {
                WalkEvent::Leaf { morton, data } => Some((*morton, data.clone())),
                _ => None,
            })
            .collect()
    }

    pub fn empties(&self) -> Vec<Morton3D> {
        self.events
            .iter()
            .filter_map(|e| match e {
                WalkEvent::Empty { morton } => Some(*morton),
                _ => None,
            })
            .collect()
    }
}

impl<Data: Clone> RecordingWalker<Data> {
    /// Number of maximum-depth voxels spanned by a node of the given depth.
    fn span(morton: Morton3D) -> u32 {
        1 << (3 * (TREE_DEPTH - morton.depth().as_usize()))
    }

    /// Total number of voxels spanned by all leaves and empty nodes. A
    /// dense walk of a whole chunk must report exactly `32^3`.
    pub fn covered_volume(&self) -> u32 {
        self.events
            .iter()
            .map(|e| match e {
                WalkEvent::Leaf { morton, .. } | WalkEvent::Empty { morton } => Self::span(*morton),
                _ => 0,
            })
            .sum()
    }

    /// Every voxel spanned by a leaf, keyed by its maximum-depth Morton
    /// code. Lets walks of differently shaped trees be compared voxel by
    /// voxel.
    pub fn leaf_voxels(&self) -> BTreeMap<u32, Data> {
        let mut voxels = BTreeMap::new();
        for (morton, data) in self.leaves() {
            let shift = 3 * (TREE_DEPTH - morton.depth().as_usize());
            let first = morton.as_u32() << shift;
            for code in first..first + Self::span(morton) {
                assert!(
                    voxels.insert(code, data.clone()).is_none(),
                    "voxel {code} reported twice"
                );
            }
        }
        voxels
    }

    /// Fails unless every branch start is closed by an end at the same
    /// node, in stack order, and every child node lies inside its parent.
    pub fn assert_well_nested(&self) {
        let mut open: Vec<Morton3D> = Vec::new();
        for event in &self.events {
            match event {
                WalkEvent::BranchStart { morton, .. } => {
                    if let Some(parent) = open.last() {
                        assert_eq!(
                            morton.depth().as_usize(),
                            parent.depth().as_usize() + 1,
                            "branch {morton:?} is not a child of {parent:?}"
                        );
                        assert_eq!(morton.parent().as_u32(), parent.as_u32());
                    }
                    open.push(*morton);
                }
                WalkEvent::BranchEnd { morton } => {
                    assert_eq!(open.pop(), Some(*morton), "mismatched branch end");
                }
                WalkEvent::Leaf { morton, .. } | WalkEvent::Empty { morton } => {
                    if let Some(parent) = open.last() {
                        assert_eq!(
                            morton.depth().as_usize(),
                            parent.depth().as_usize() + 1,
                            "node {morton:?} is not a child of {parent:?}"
                        );
                        assert_eq!(morton.parent().as_u32(), parent.as_u32());
                    }
                }
            }
        }
        assert!(open.is_empty(), "unclosed branches: {open:?}");
    }
}

impl<Data: Clone> Walker<Data> for RecordingWalker<Data> {
    fn on_branch_start(&mut self, morton: Morton3D, children_mask: u8) {
        self.events.push(WalkEvent::BranchStart {
            morton,
            mask: children_mask,
        });
    }

    fn on_branch_end(&mut self, morton: Morton3D) {
        self.events.push(WalkEvent::BranchEnd { morton });
    }

    fn on_leaf(&mut self, morton: Morton3D, data: &Data) {
        self.events.push(WalkEvent::Leaf {
            morton,
            data: data.clone(),
        });
    }
}

impl<Data: Clone> DenseWalker<Data> for RecordingWalker<Data> {
    fn on_branch_start(&mut self, morton: Morton3D, children_mask: u8) {
        Walker::on_branch_start(self, morton, children_mask);
    }

    fn on_branch_end(&mut self, morton: Morton3D) {
        Walker::on_branch_end(self, morton);
    }

    fn on_leaf(&mut self, morton: Morton3D, data: &Data) {
        Walker::on_leaf(self, morton, data);
    }

    fn on_empty(&mut self, morton: Morton3D) {
        self.events.push(WalkEvent::Empty { morton });
    }
}
