use crate::axis::Axis;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceLayer {
    pub axis: Axis,
    pub layer: usize,
}

impl SurfaceLayer {
    pub fn new(axis: Axis, layer: usize) -> SurfaceLayer {
        SurfaceLayer { axis, layer }
    }
}
