pub mod axis;
pub mod face;
pub mod meshing;
pub mod utils;

mod coords_2d;
mod coords_3d;
mod depth;
mod morton_2d;
mod morton_3d;
mod octree;
mod walking;

pub use coords_2d::Coords2D;
pub use coords_3d::Coords3D;
pub use depth::Depth;
pub use morton_2d::Morton2D;
pub use morton_3d::Morton3D;
pub use octree::{CompactOctree, Fixed8Octree};
pub use walking::{DenseWalk, DenseWalker, Walk, Walker};

pub type ChunkCoord = u8;

#[cfg(test)]
mod tests;
