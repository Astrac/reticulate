mod morton;
mod octree;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords {
    x: u8,
    y: u8,
    z: u8,
}

pub use morton::Morton;
pub use octree::Octree;

#[cfg(test)]
mod tests;
