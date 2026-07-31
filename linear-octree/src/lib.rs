mod morton;
mod octree;
mod block_faces;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

pub use morton::Morton;
pub use octree::Octree;
pub use block_faces::BlockFaces;

#[cfg(test)]
mod tests;
