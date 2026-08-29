mod compact;
mod fixed8;

pub use compact::{CompactOctree, CompactOctreeNode};
pub use fixed8::{Fixed8Octree, Fixed8OctreeNode};

pub const TREE_DEPTH: usize = 5;
