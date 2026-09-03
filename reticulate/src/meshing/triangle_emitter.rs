use crate::{axis::*, meshing::Triangle};

#[derive(Debug, Default)]
pub struct TriangleEmitter;

impl TriangleEmitter {
    pub fn process<F>(
        &self,
        axis: Axis,
        layer: usize,
        fill_mask: &[u32],
        material: u16,
        on_triangle: &F,
    ) where
        F: FnMut(Triangle),
    {
    }
}
