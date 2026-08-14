mod faces;
mod morton;
pub mod octree;
mod quadtree;
mod sketches;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

pub use faces::BlockFaces;
pub use morton::Morton;
pub use octree::{CompactOctree, CompactOctreeNode, Fixed8Octree, Fixed8OctreeNode, Walk, Walker};

#[cfg(test)]
mod tests;
