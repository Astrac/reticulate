mod face_walker;
mod faces;
mod morton;
mod octree;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

pub use faces::BlockFaces;
pub use morton::Morton;
pub use octree::{CompactOctree, CompactOctreeNode, Fixed8Octree, Fixed8OctreeNode, Walk, Walker};
pub use face_walker::FaceWalker;

#[cfg(test)]
mod tests;
