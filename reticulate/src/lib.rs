mod axis;
mod coords_2d;
mod coords_3d;
mod coords_lut;
mod definitions;
mod depth;
mod face;
mod meshing;
mod morton_2d;
mod morton_3d;
mod morton_lut;
mod octree;
mod utils;
mod walking;

pub use axis::Axis;
pub use coords_2d::Coords2D;
pub use coords_3d::Coords3D;
pub use definitions::*;
pub use depth::Depth;
pub use face::Face;
pub use meshing::Mesher;
pub use morton_2d::Morton2D;
pub use morton_3d::Morton3D;
pub use octree::{CompactOctree, Fixed8Octree};
pub use walking::{DenseWalk, DenseWalker, Walk, Walker};

pub type ChunkCoord = u8;
pub type CellMask = u32;
pub type BoundaryMask = u64;

#[cfg(test)]
mod tests;
