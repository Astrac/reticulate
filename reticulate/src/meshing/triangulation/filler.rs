use crate::{
    Coords2D, PLANE_SIDE,
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
    pub fn new(claims: &'a Claims, parity: &'a Parity<'a>) -> Self {
        Filler { claims, parity }
    }

    pub fn fill_region(
        &self,
        layer: SurfaceLayer,
        edges: &Edges,
        seed: Coords2D,
    ) -> [u32; PLANE_SIDE] {
        let mut region = [0u32; PLANE_SIDE];
        let seed_row = seed.u as usize; // row: u, column: v
        region[seed_row] = self.expand(layer, edges, seed_row, 1 << seed.v);

        loop {
            let mut changed = false;
            for to in 1..PLANE_SIDE {
                changed |= self.grow(layer, edges, &mut region, to - 1, to);
            }
            for to in (0..PLANE_SIDE - 1).rev() {
                changed |= self.grow(layer, edges, &mut region, to + 1, to);
            }
            if !changed {
                return region;
            }
        }
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

    fn expand(&self, layer: SurfaceLayer, edges: &Edges, row: usize, columns: u32) -> u32 {
        let walls =
            edges.walls(row) | bitmasks::segment_boundaries(self.claims.claimed_row(layer, row));
        Runs::from_blocks(walls, columns).fill_mask()
    }
}
