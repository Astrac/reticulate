use crate::{
    CellMask, Coords2D, PLANE_SIDE,
    meshing::{
        bitfields::{Claims, Parity},
        surface_layer::SurfaceLayer,
        triangulation::{edges::Edges, runs::Runs},
    },
    utils::bitmasks,
};

pub struct Filler<'a> {
    claims: &'a Claims<'a>,
    parity: &'a Parity<'a>,
}

impl<'a> Filler<'a> {
    #[inline(always)]
    pub fn new(claims: &'a Claims, parity: &'a Parity<'a>) -> Self {
        Filler { claims, parity }
    }

    pub fn fill_region(
        &self,
        layer: SurfaceLayer,
        edges: &Edges,
        seed: Coords2D,
    ) -> [u32; PLANE_SIDE] {
        crate::start_timer!(timer_fill_region);
        let mut region = [0u32; PLANE_SIDE];
        let seed_row = seed.u as usize; // row: u, column: v
        region[seed_row] = self.expand(layer, edges, seed_row, 1 << seed.v);

        let mut dirty: u32 = 1 << seed_row;
        while dirty != 0 {
            let row = dirty.trailing_zeros() as usize;
            dirty &= dirty - 1;
            if row > 0 && self.grow(layer, edges, &mut region, row, row - 1) {
                dirty |= 1 << (row - 1);
            }
            if row + 1 < PLANE_SIDE && self.grow(layer, edges, &mut region, row, row + 1) {
                dirty |= 1 << (row + 1);
            }
        }

        region
    }

    fn grow(
        &self,
        layer: SurfaceLayer,
        edges: &Edges,
        region: &mut [u32; PLANE_SIDE],
        from: usize,
        to: usize,
    ) -> bool {
        let source = region[from] & !region[to];
        if source == 0 {
            return false;
        }
        // Parity bit b is the transition between rows b - 1 and b.
        let boundary = from.max(to);
        let columns = source
            & !self.parity.boundary_guard(layer, boundary, source)
            & !self.claims.claimed_row(layer, to);
        if columns == 0 {
            return false;
        }
        region[to] |= self.expand(layer, edges, to, columns);
        true
    }

    fn expand(
        &self,
        layer: SurfaceLayer,
        edges: &Edges,
        row: usize,
        columns: CellMask,
    ) -> CellMask {
        let walls =
            edges.walls(row) | bitmasks::segment_boundaries(self.claims.claimed_row(layer, row));
        Runs::from_blocks(walls, columns).fill_mask()
    }
}
