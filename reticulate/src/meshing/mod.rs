mod mesher;
mod triangle;

pub(crate) mod bitfields;
pub(crate) mod gathering_walker;
pub(crate) mod surface_layer;
pub(crate) mod surface_scanner;
pub(crate) mod surface_walker;
pub(crate) mod triangulation;

pub use mesher::Mesher;
pub use triangle::Triangle;

// TODO - Support this or delete it
#[allow(unused)]
pub mod packed_triangle;
