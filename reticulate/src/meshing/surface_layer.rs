use crate::{Axis, ChunkCoord};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceLayer {
    pub axis: Axis,
    pub index: u8,
}

impl SurfaceLayer {
    #[inline(always)]
    pub const fn new(axis: Axis, index: ChunkCoord) -> SurfaceLayer {
        SurfaceLayer { axis, index }
    }
}
