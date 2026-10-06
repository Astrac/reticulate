use crate::{
    BoundaryMask, PLANE_SIDE,
    meshing::{bitfields::Parity, surface_layer::SurfaceLayer},
};

// These are the parity violations forced at the edges of the chunk
// in order to force matching vertices and continuity across chunks
pub const BOUNDARY_WALLS: BoundaryMask = 1 | (1 << PLANE_SIDE);

pub struct Edges<'a> {
    current: &'a [u32],
    previous: Option<&'a [u32]>,
}

impl<'a> Edges<'a> {
    #[inline(always)]
    pub fn from_parity(parity: &'a Parity<'a>, layer: SurfaceLayer) -> Edges<'a> {
        let rows_axis = layer.axis.next().next();
        Edges {
            current: parity.gather_edges(SurfaceLayer::new(rows_axis, layer.index)),
            previous: (layer.index > 0)
                .then(|| parity.gather_edges(SurfaceLayer::new(rows_axis, layer.index - 1))),
        }
    }

    pub fn walls(&self, row: usize) -> BoundaryMask {
        (self.current[row] | self.previous.map_or(0, |p| p[row])) as BoundaryMask | BOUNDARY_WALLS
    }
}
