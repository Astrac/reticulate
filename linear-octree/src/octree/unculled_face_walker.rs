use crate::{face::*, morton::Morton, octree::Walker};

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
    F: FnMut(Morton, u8, Face),
{
    fn on_branch_start(&mut self, _depth: u8, _morton: Morton, _children_mask: u8) {}

    fn on_branch_end(&mut self, _depth: u8, _morton: Morton) {}

    fn on_leaf(&mut self, depth: u8, morton: Morton, _data: &Data) {
        (self.on_face)(morton, depth, FACE_X_N);
        (self.on_face)(morton, depth, FACE_X_P);
        (self.on_face)(morton, depth, FACE_Y_N);
        (self.on_face)(morton, depth, FACE_Y_P);
        (self.on_face)(morton, depth, FACE_Z_N);
        (self.on_face)(morton, depth, FACE_Z_P);
    }
}
