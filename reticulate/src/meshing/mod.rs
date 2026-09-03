mod compacting_face_walker;
mod decision_boundaries;
mod harvest;
mod mesh_face;
mod mesher;
mod parity_walker;
mod shape_walker;
mod simple_face_walker;
mod triangle_emitter;
mod unculled_face_walker;

pub mod bitfields;
pub mod face;

pub use compacting_face_walker::CompactingFaceWalker;
pub use mesh_face::MeshFace;
pub use simple_face_walker::SimpleFaceWalker;

use crate::{Coords3D, meshing::bitfields::MaterialId};

#[derive(Clone, Debug, Default)]
pub struct Triangle {
    vertices: [Coords3D; 3],
    material: MaterialId,
}
