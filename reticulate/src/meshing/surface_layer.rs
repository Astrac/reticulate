use crate::{ChunkCoord, axis::Axis};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceLayer {
    pub axis: Axis,
    pub layer: u8,
}

impl SurfaceLayer {
    #[inline(always)]
    pub const fn new(axis: Axis, layer: ChunkCoord) -> SurfaceLayer {
        SurfaceLayer { axis, layer }
    }
}
