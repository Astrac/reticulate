mod block_faces;
pub mod faces;
mod morton;
mod octree;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

pub use block_faces::BlockFaces;
pub use morton::Morton;
pub use octree::{CompactOctree, CompactOctreeNode, Fixed8Octree, Fixed8OctreeNode};

#[cfg(test)]
mod tests;
