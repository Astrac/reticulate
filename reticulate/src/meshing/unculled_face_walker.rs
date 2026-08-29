use crate::{Depth, face::*, morton_3d::Morton3D, walking::Walker};

// TODO: publish this or strip it

#[allow(dead_code)]
pub struct UnculledFaceWalker<F> {
    on_face: F,
}

#[allow(dead_code)]
impl<F> UnculledFaceWalker<F> {
    pub fn new(f: F) -> UnculledFaceWalker<F> {
        UnculledFaceWalker { on_face: f }
    }
}

impl<Data, F> Walker<Data> for UnculledFaceWalker<F>
where
    F: FnMut(Morton3D, Depth, Face),
{
    fn on_branch_start(&mut self, _depth: Depth, _morton: Morton3D, _children_mask: u8) {}

    fn on_branch_end(&mut self, _depth: Depth, _morton: Morton3D) {}

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, _data: &Data) {
        (self.on_face)(morton, depth, FACE_X_N);
        (self.on_face)(morton, depth, FACE_X_P);
        (self.on_face)(morton, depth, FACE_Y_N);
        (self.on_face)(morton, depth, FACE_Y_P);
        (self.on_face)(morton, depth, FACE_Z_N);
        (self.on_face)(morton, depth, FACE_Z_P);
    }
}
