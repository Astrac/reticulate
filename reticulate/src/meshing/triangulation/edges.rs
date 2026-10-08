use crate::{
    BoundaryMask, PLANE_SIDE,
    meshing::{bitfields::Parity, surface_layer::SurfaceLayer},
};

// These are the parity violations forced at the edges of the chunk
// in order to force matching vertices and continuity across chunks
pub const BOUNDARY_WALLS: BoundaryMask = 1 | (1 << PLANE_SIDE);
const ZERO_ROWS: [u32; PLANE_SIDE] = [0; PLANE_SIDE];

pub struct Edges<'a> {
    current: &'a [u32],
    previous: &'a [u32],
}

impl<'a> Edges<'a> {
    #[inline(always)]
    pub fn from_parity(parity: &'a Parity<'a>, layer: SurfaceLayer) -> Edges<'a> {
        let rows_axis = layer.axis.prev();

        let previous = if layer.index > 0 {
            parity.gather_edges(SurfaceLayer::new(rows_axis, layer.index - 1))
        } else {
            &ZERO_ROWS
        };

        Edges {
            current: parity.gather_edges(SurfaceLayer::new(rows_axis, layer.index)),
            previous,
        }
    }

    #[inline(always)]
    pub fn walls(&self, row: usize) -> BoundaryMask {
        (self.current[row] | self.previous[row]) as BoundaryMask | BOUNDARY_WALLS
    }
}
